use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use derive_builder::Builder;

#[allow(unused)]
#[derive(Builder, Debug, Default)]
#[builder(build_fn(name = "mybuild"))]
struct User {
    // 实现 from 方法，使得我们可以直接传递字符串字面量
    #[builder(setter(into))]
    name: String,
    age: u32,
    // strip_option 使你能够直接传递非 Option 类型的值，这样就不需要手动包装在 Some 中了
    // default 使得 email 字段在 build 时可以不传递，而使用默认值 None, 我们使用 #[derive(Default)] 为 User 结构体实现了 Default trait
    #[builder(setter(into, strip_option), default)]
    email: Option<String>,
    // 设置 country 字段的默认值为 "China"
    #[builder(setter(into), default = "String::from(\"China\")")]
    country: String,
    // 设置 height 字段的默认值为 180
    #[builder(default = "180")]
    height: u32,
    // skills 字段是一个 Vec<String> 类型，可以通过 skill 方法多次添加元素
    #[builder(default = "vec![]", setter(each(name = "skill", into)))]
    skills: Vec<String>,
    #[builder(setter(custom))]
    dob: DateTime<Utc>,
    #[builder(setter(skip))]
    calculate_age: u32,
}

impl UserBuilder {
    // 根据 dob 字段的值计算年龄，并将其设置为 calculateAge 字段的值
    pub fn build(&self) -> Result<User> {
        let mut user = self.mybuild()?;
        user.calculate_age = (Utc::now().year() - user.dob.year()) as _;
        Ok(user)
    }
    // 受一个字符串参数 value，尝试将其解析为 RFC 3339 格式的日期时间，然后将其转换为 UTC 时间，并设置为 dob 字段的值。
    // 如果解析失败，dob 字段的值将被设置为 None。
    // Self 表示 UserBuilder 类型本身，&mut Self 表示一个可变引用。
    pub fn dob(&mut self, value: &str) -> &mut Self {
        self.dob = DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.with_timezone(&Utc))
            // 如果解析失败（例如，value 不是一个有效的 RFC 3339 日期时间），
            // parse_from_rfc3339 方法将返回一个 Err，ok 方法将将其转换为 None。
            .ok();
        self
    }
}

fn main() -> Result<()> {
    let user = UserBuilder::default()
        .name("Alice")
        .age(30)
        .email("seven@example.com")
        .skill("programming")
        .skill("debugging")
        .dob("1990-01-01T00:00:00Z")
        .build()?;

    // User { name: "Alice", age: 30, email: Some("seven@example.com"), country: "China", height: 180, skills: ["programming", "debugging"], dob: 1990-01-01T00:00:00Z, calculate_age: 34 }
    println!("{:?}", user);

    Ok(())
}
