#[derive(Debug)]
pub struct UserId(i32);

impl From<i32> for UserId {
    fn from(id: i32) -> Self {
        UserId(id)
    }
}

impl From<UserId> for i32 {
    fn from(user_id: UserId) -> i32 {
        user_id.0
    }
}
