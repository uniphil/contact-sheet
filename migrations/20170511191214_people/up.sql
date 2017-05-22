CREATE TABLE people (
    PRIMARY KEY (id),
    id          UUID        DEFAULT uuid_generate_v4()  NOT NULL,
    created     TIMESTAMP   DEFAULT now()               NOT NULL,
    email       CITEXT                                  NOT NULL UNIQUE,
                CONSTRAINT valid_email
                CHECK (email LIKE '%_@_%'),
    address     ADDRESS                                 NULL,
    customer    TEXT                                    NULL UNIQUE
)
