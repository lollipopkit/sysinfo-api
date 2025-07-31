/// Create a successful JSON response
#[macro_export]
macro_rules! json_resp_ok {
    ($data:expr) => {
        Json(Resp::success($data))
    };
}

/// Create an error JSON response
#[macro_export]
macro_rules! json_resp_err {
    ($code:expr, $msg:expr) => {
        Json(Resp::error($code, $msg))
    };
    ($code:expr) => {
        Json(Resp::error($code, "Internal server error"))
    };
}