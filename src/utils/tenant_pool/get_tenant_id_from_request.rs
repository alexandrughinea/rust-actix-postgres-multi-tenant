use actix_web::{HttpRequest, HttpResponse};
use uuid::Uuid;

pub fn get_tenant_id_from_request(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    match req.headers().get("x-tenant-id") {
        Some(tenant_id) => {
            match tenant_id.to_str() {
                Ok(tenant_id_str) => match Uuid::parse_str(tenant_id_str) {
                    Ok(uuid) => Ok(uuid),
                    Err(_) => Err(HttpResponse::BadRequest()
                        .body("Invalid UUID format in x-tenant-id header")),
                },
                Err(_) => Err(HttpResponse::BadRequest().body("Invalid x-tenant-id header")),
            }
        }
        None => Err(HttpResponse::BadRequest().body("x-tenant-id header missing")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::body::MessageBody;
    use actix_web::test::TestRequest;
    use uuid::Uuid;

    #[test]
    fn test_valid_tenant_id() {
        let valid_uuid = Uuid::new_v4();
        let req = TestRequest::default()
            .insert_header(("x-tenant-id", valid_uuid.to_string()))
            .to_http_request();

        let result = get_tenant_id_from_request(&req);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_uuid);
    }

    #[test]
    fn test_missing_tenant_id() {
        let req = TestRequest::default().to_http_request();

        let result = get_tenant_id_from_request(&req);

        assert!(result.is_err());
        let error_response = result.unwrap_err();
        assert_eq!(
            error_response.status(),
            actix_web::http::StatusCode::BAD_REQUEST
        );

        // Convert BoxBody to String for comparison
        let body = error_response.into_body().try_into_bytes().unwrap();
        assert_eq!(body, "x-tenant-id header missing");
    }

    #[test]
    fn test_invalid_uuid_format() {
        let req = TestRequest::default()
            .insert_header(("x-tenant-id", "not-a-uuid"))
            .to_http_request();

        let result = get_tenant_id_from_request(&req);

        assert!(result.is_err());
        let error_response = result.unwrap_err();
        assert_eq!(
            error_response.status(),
            actix_web::http::StatusCode::BAD_REQUEST
        );

        let body = error_response.into_body().try_into_bytes().unwrap();
        assert_eq!(body, "Invalid UUID format in x-tenant-id header");
    }

    #[test]
    fn test_invalid_header_value() {
        let req = TestRequest::default()
            .insert_header(("x-tenant-id", vec![0xFF, 0xFF, 0xFF]))
            .to_http_request();

        let result = get_tenant_id_from_request(&req);

        assert!(result.is_err());
        let error_response = result.unwrap_err();
        assert_eq!(
            error_response.status(),
            actix_web::http::StatusCode::BAD_REQUEST
        );

        let body = error_response.into_body().try_into_bytes().unwrap();
        assert_eq!(body, "Invalid x-tenant-id header");
    }
}
