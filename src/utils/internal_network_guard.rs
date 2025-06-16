use actix_web::{
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use std::pin::Pin;
use std::{
    future::{ready, Ready},
    net::IpAddr,
    rc::Rc,
    str::FromStr,
};

pub struct InternalNetworkGuard {
    trusted_networks: Vec<String>,
    trusted_headers: Vec<String>,
}

impl Default for InternalNetworkGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalNetworkGuard {
    pub fn new() -> Self {
        InternalNetworkGuard {
            // Default trusted networks (localhost, private networks)
            trusted_networks: vec![
                "127.0.0.1".to_string(), // localhost
                "::1".to_string(),       // IPv6 localhost
                "10.".to_string(),       // Private Class A
                "172.16.".to_string(),   // Private Class B start
                "172.31.".to_string(),   // Private Class B end
                "192.168.".to_string(),  // Private Class C
            ],
            // Headers that might indicate trusted internal service calls
            trusted_headers: vec![
                "X-Internal-Request".to_string(),
                "X-Service-Token".to_string(),
            ],
        }
    }
    pub fn with_networks(mut self, networks: Vec<String>) -> Self {
        self.trusted_networks.extend(networks);
        self
    }
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.trusted_headers.extend(headers);
        self
    }

    /// Check if an IP address is in a trusted network
    pub fn is_trusted_ip(&self, ip_str: &str) -> bool {
        if let Ok(ip) = IpAddr::from_str(ip_str) {
            if ip.to_string() == "127.0.0.1" || ip.to_string() == "::1" {
                return true;
            }

            for network in &self.trusted_networks {
                if ip_str.starts_with(network) {
                    return true;
                }
            }
        }
        false
    }
}

impl<S, B> Transform<S, ServiceRequest> for InternalNetworkGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InternalNetworkGuardMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InternalNetworkGuardMiddleware {
            service: Rc::new(service),
            trusted_networks: self.trusted_networks.clone(),
            trusted_headers: self.trusted_headers.clone(),
        }))
    }
}

pub struct InternalNetworkGuardMiddleware<S> {
    service: Rc<S>,
    trusted_networks: Vec<String>,
    trusted_headers: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for InternalNetworkGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let trusted_headers = self.trusted_headers.clone();
        let trusted_networks = self.trusted_networks.clone();

        let mut is_trusted = false;

        for header_name in &trusted_headers {
            if req.headers().contains_key(header_name) {
                is_trusted = true;
                break;
            }
        }

        // If not already trusted by headers, check IP address
        if !is_trusted {
            let remote_ip = req
                .connection_info()
                .realip_remote_addr()
                .map(|addr| addr.to_string());

            if let Some(addr) = remote_ip {
                let ip = addr.split(':').next().unwrap_or(&addr);
                let guard = InternalNetworkGuard {
                    trusted_networks: trusted_networks.clone(),
                    trusted_headers: vec![],
                };

                if guard.is_trusted_ip(ip) {
                    is_trusted = true;
                }
            }
        }

        Box::pin(async move {
            if is_trusted {
                srv.call(req).await
            } else {
                let error_msg = serde_json::json!({
                    "error": "Access to internal endpoints is restricted to trusted networks",
                    "code": "FORBIDDEN_INTERNAL_ACCESS"
                });
                Err(actix_web::error::ErrorForbidden(error_msg.to_string()))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::InternalNetworkGuard;

    #[test]
    fn test_is_trusted_ip() {
        let guard = InternalNetworkGuard::new();

        assert!(guard.is_trusted_ip("127.0.0.1"));
        assert!(guard.is_trusted_ip("::1"));
        assert!(guard.is_trusted_ip("10.0.0.1"));
        assert!(guard.is_trusted_ip("192.168.1.1"));

        assert!(!guard.is_trusted_ip("8.8.8.8"));
        assert!(!guard.is_trusted_ip("203.0.113.1"));
    }
}
