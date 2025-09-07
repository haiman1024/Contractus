// Contractus 集成测试
// 测试整个编译流程：词法分析 + 语法分析

use contractus::{Lexer, Parser};

fn parse_program(input: &str) -> Result<contractus::ast::Program, Vec<contractus::parser::ParseError>> {
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| {
        vec![contractus::parser::ParseError::new(
            format!("Lexer error: {:?}", e),
            contractus::span::Span::new(0, 0, 1, 1),
        )]
    })?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_hello_world() {
    let input = r#"
        fn main() -> i32 {
            print(42);
            return 0;
        }
    "#;
    
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_struct_and_function_example() {
    let input = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        fn add_points(a: Point, b: Point) -> Point {
            let result: Point = Point {
                x: a.x + b.x,
                y: a.y + b.y,
            };
            return result;
        }

        fn main() -> i32 {
            let p1 = Point { x: 10, y: 20 };
            let p2 = Point { x: 30, y: 40 };
            let sum = add_points(p1, p2);

            print(sum.x);  // 输出 40
            return 0;
        }
    "#;
    
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_for_loop_example() {
    let input = r#"
        fn sum_array(arr: [i32; 5]) -> i32 {
            let total = 0;
            for item in arr {
                total = total + item;
            }
            return total;
        }

        fn main() -> i32 {
            // 范围循环
            for i in 0..10 {
                print(i);
            }

            // 数组遍历
            let numbers: [i32; 5] = [1, 2, 3, 4, 5];
            let sum = sum_array(numbers);

            return 0;
        }
    "#;
    
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_complex_program() {
    let input = r#"
        // 复杂程序测试所有特性
        
        struct Point {
            x: i32,
            y: i32,
        }
        
        struct Rectangle {
            top_left: Point,
            bottom_right: Point,
        }
        
        enum Shape {
            Circle(Point, i32),
            Rectangle(Rectangle),
        }
        
        fn area(shape: Shape) -> i32 {
            match shape {
                Circle(center, radius) => 3 * radius * radius,
                Rectangle(rect) => {
                    let width = rect.bottom_right.x - rect.top_left.x;
                    let height = rect.top_left.y - rect.bottom_right.y;
                    return width * height;
                }
            }
        }
        
        fn main() -> i32 {
            let circle = Shape::Circle(Point { x: 0, y: 0 }, 10);
            let rect = Shape::Rectangle(Rectangle {
                top_left: Point { x: 0, y: 10 },
                bottom_right: Point { x: 10, y: 0 },
            });
            
            print(area(circle));
            print(area(rect));
            
            // 测试for循环
            for i in 0..5 {
                print(i * 2);
            }
            
            // 测试数组
            let values: [i32; 3] = [1, 2, 3];
            for val in values {
                print(val * val);
            }
            
            return 0;
        }
    "#;
    
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_generic_example() {
    let input = r#"
        struct Stack<T> {
            data: [T; 100],
            size: i32,
        }
        
        impl<T> Stack<T> {
            fn new() -> Stack<T> {
                return Stack {
                    data: [/* zero-initialized */],
                    size: 0,
                };
            }
            
            fn push(mut self, value: T) {
                self.data[self.size] = value;
                self.size = self.size + 1;
            }
            
            fn pop(mut self) -> T {
                self.size = self.size - 1;
                return self.data[self.size];
            }
            
            fn is_empty(self) -> bool {
                return self.size == 0;
            }
        }
        
        fn main() -> i32 {
            let mut stack = Stack::<i32>::new();
            
            stack.push(1);
            stack.push(2);
            stack.push(3);
            
            while !stack.is_empty() {
                print(stack.pop());
            }
            
            return 0;
        }
    "#;
    
    assert!(parse_program(input).is_ok());
}