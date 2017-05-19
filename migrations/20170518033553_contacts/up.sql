CREATE TABLE contacts (
    PRIMARY KEY (id),
    id      UUID        DEFAULT uuid_generate_v4()  NOT NULL,
    created TIMESTAMP   DEFAULT now()               NOT NULL,
    account UUID                                    NOT NULL
            REFERENCES people(id)
                    ON DELETE CASCADE
                    ON UPDATE CASCADE,
    name    TEXT                                    NOT NULL,
    info    TEXT                                    NOT NULL
)
