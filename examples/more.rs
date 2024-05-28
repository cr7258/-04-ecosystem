use derive_more::{Add, Constructor, Deref, DerefMut, Display, From, Sub};

// 实现 Add 和 Sub trait，允许我们对 Point 类型的实例进行加法和减法操作
#[derive(Add, Sub, Debug, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[allow(unused)]
// 实现 From trait，允许我们将 i32 类型的值转换为 Age 类型的值
#[derive(From, Debug)]
struct Age(i32);

// 实现 Display trait，允许我们自定义 MyEnum 类型的实例的显示方式
#[derive(Display)]
enum MyEnum {
    #[display(fmt = "int: {}", _0)]
    Int(i32),
    #[display(fmt = "nothing")]
    Nothing,
}

#[allow(unused)]
// 实现 Constructor trait，允许我们使用 new 方法创建 MyStruct 类型的实例
#[derive(Constructor, Debug)]
struct MyStruct {
    x: i32,
    y: i32,
}

// Deref：允许你通过 &T（不可变引用）访问内部数据。主要用于实现不可变解引用。
// DerefMut：允许你通过 &mut T（可变引用）访问内部数据。主要用于实现可变解引用。
#[derive(Deref, DerefMut, Debug)]
struct MyVec(Vec<i32>);

fn main() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 3, y: 4 };

    let p3 = p1 + p2;
    let p4 = p3 - p1;

    println!("{:?}", p3); // Point { x: 4, y: 6 }
    println!("{:?}", p4); // Point { x: 3, y: 4 }

    let my_age: Age = 30.into();
    println!("{:?}", my_age); // Age(30)

    let myenum = MyEnum::Int(10);
    println!("{}", myenum); // int: 10
    let myenum2 = MyEnum::Nothing;
    println!("{}", myenum2); // nothing

    let p = MyStruct::new(1, 2);
    println!("{:?}", p); // Point { x: 1, y: 2 }

    let mut my_vec = MyVec(vec![1, 2, 3]);
    // 使用 Deref 实现自动解引用
    println!("Length: {}", my_vec.len());
    // 使用 DerefMut 实现可变解引用
    my_vec.push(4);
    // 验证元素是否成功添加
    println!("{:?}", my_vec);
}
