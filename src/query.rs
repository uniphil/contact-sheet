#[macro_export]
macro_rules! write {
    ($db:ident, $q:expr, $( $p:expr ),* ) => ({
        let n = $db.conn()
            .execute($q, &[ $($p),* ])
            .chain_err(|| "Error while talking to the database")?;
        if n != 1 {
            bail!(format!("Expected to touch one row, but touched {}", n));
        }
    })
}

#[macro_export]
macro_rules! filter {
    ($db:ident, $q:expr) => (
        filter!($db, $q,)
    );
    ($db:ident, $q:expr, $( $p:expr ),* ) => (
        $db.conn()
            .query($q, &[ $($p),* ])
            .chain_err(|| "Error while talking to the database")?
            .into_iter()
    );
}

#[macro_export]
macro_rules! find {
    ($db:ident, $q:expr) => (
        find!($db, $q,)
    );
    ($db:ident, $q:expr, $( $p:expr ),* ) => (
        $db.conn()
            .query($q, &[ $($p),* ])
            .chain_err(|| "Error while talking to the database")
            .and_then(|rows| match rows.len() {
                0|1 => Ok(rows),
                _ => Err("Expected 0 or 1 rows, found more".into()),
            })?
            .into_iter()
            .next()
    );
}
