macro_rules! read_lock {
    ($e:expr) => {
        ($e).read().map_err(|err| ErrorCode::Unexpected {
            error_message: format!("Error acquiring read lock: {}", err),
        })
    };
}

macro_rules! write_lock {
    ($e:expr) => {
        ($e).write().map_err(|err| ErrorCode::Unexpected {
            error_message: format!("Error acquiring write lock: {}", err),
        })
    };
}

macro_rules! read_pool {
    ($e:expr) => {
        ($e).read().await.as_ref().ok_or(ErrorCode::Unexpected {
            error_message: format!("Pool is already closed"),
        })
    };
}

macro_rules! read_req {
    ($e:expr) => {
        ($e).blocking_read().as_ref().ok_or(ErrorCode::Unexpected {
            error_message: format!("Request is already used"),
        })
    };
}

macro_rules! write_req {
    ($e:expr) => {
        ($e).blocking_write().as_mut().ok_or(ErrorCode::Unexpected {
            error_message: format!("Request is already used"),
        })
    };
}

macro_rules! take_req {
    ($e:expr) => {
        ($e).write().await.take().ok_or(ErrorCode::Unexpected {
            error_message: format!("Request is already used"),
        })
    };
}
