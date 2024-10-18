graph LR
subgraph Clients
A[Client from tenant A]
B[Client from tenant B]
C[Client from tenant C]
end

    subgraph Application
        TA[Tenant A context]
        TB[Tenant B context]
        TC[Tenant C context]
    end

    subgraph DB_Connections
        DBA[DB connection for tenant A]
        DBB[DB connection for tenant B]
        DBC[DB connection for tenant C]
    end

    subgraph Database
        DA[row data for tenant A]
        DB[row data for tenant B]
        DC[row data for tenant C]
    end

    A --> TA
    B --> TB
    C --> TC

    TA --> DBA
    TB --> DBB
    TC --> DBC

    DBA --> DA
    DBB --> DB
    DBC --> DC