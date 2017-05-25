CREATE TABLE cards (
    PRIMARY KEY (id),
    id          TEXT                        NOT NULL,
    created     TIMESTAMP   DEFAULT now()   NOT NULL,
    brand       ccbrand                     NOT NULL,
    country     VARCHAR(2)                  NOT NULL,
    customer    TEXT                        NOT NULL
                REFERENCES people(customer)
                        ON DELETE CASCADE
                        ON UPDATE CASCADE,
    last4       VARCHAR(4)                  NOT NULL,
                CONSTRAINT valid_last_4
                     CHECK (last4 SIMILAR TO '[[:digit:]]{4}'),
    name        TEXT                        NOT NULL
);
