use chrono::{DateTime, Utc};

pub struct User
{
    username: String,
    user_id: u64,
    join_date: DateTime<Utc>,
}

impl Default for User
{
    fn default() -> Self
    {
        User
        {
            username: String::new(),
            user_id: 0,
            join_date: DateTime::default()
        }
    }
}
