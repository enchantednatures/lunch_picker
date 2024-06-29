#[derive(Debug)]
pub struct UserId(i32);

impl UserId {
    pub fn as_i32(&self) -> &i32 {
        &self.0
    }
}

/*

use thiserror::Error;
#[derive(Error, Debug)]
enum UserIdValidationError{}


impl TryFrom<i32> for UserId {
    type Error = UserIdValidationError;
    fn try_from(id: i32) -> Result<Self, Self::Error> {
        if(id == 0){
            return Self::Error
        }
        Ok(UserId(id))
    }
}

*/

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
