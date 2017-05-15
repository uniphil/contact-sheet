create table sessions (
    PRIMARY KEY (login_key),
    login_key   UUID        DEFAULT uuid_generate_v4()  NOT NULL,
    created     TIMESTAMP   DEFAULT now()               NOT NULL,
    account     UUID                                    NOT NULL
                REFERENCES people (id)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE,
    -- login_ip    INET                                    NOT NULL,
    -- login_ua    TEXT                                    NOT NULL,
    session_id  UUID                                    NULL UNIQUE,
    accessed    TIMESTAMP                               NULL
)
