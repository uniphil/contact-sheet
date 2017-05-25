-- http://localhost:8000/login?key=ab69fbd6-1b9f-4bd7-9342-c4a60aa62a9d
CREATE TYPE ccbrand AS ENUM (
    'Visa',
    'American Express',
    'MasterCard',
    'Discover',
    'JCB',
    'Diners Club',
    'Unknown'
)
