// main.rs
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc; // Rc 用于共享所有权，完美模拟闭包对环境的共享 // RefCell 用于内部可变性，允许我们在不可变上下文中修改环境

// 1. JS 值的枚举
#[derive(Clone, Debug)]
enum JsValue {
    Undefined,
    Number(f64),
    String(String),
    // 函数需要捕获它被创建时的外部环境
    Function(FunctionBody),
}

// 定义函数体，它需要知道自己的外部作用域
#[derive(Clone)]
struct FunctionBody {
    // 函数创建时捕获的外部环境
    outer_env: Rc<LexicalEnvironment>,
    // 用 Rust 的闭包来模拟 JS 函数的实际代码
    // Box<dyn Fn..> 是一个动态分派的闭包
    // 传入引擎引用，以便在函数体内可以调用引擎功能（如解析变量）
    body: Rc<Box<dyn Fn(&mut JsEngine)>>,
}

// 为 FunctionBody 实现 Debug trait，以便打印
impl std::fmt::Debug for FunctionBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Function]")
    }
}

// 2. 词法环境
#[derive(Debug, Clone)]
struct LexicalEnvironment {
    // 存储变量的地方
    bindings: RefCell<HashMap<String, JsValue>>,
    // 指向外部作用域的链接
    outer: Option<Rc<LexicalEnvironment>>,
}

impl LexicalEnvironment {
    fn new(outer: Option<Rc<LexicalEnvironment>>) -> Self {
        LexicalEnvironment {
            bindings: RefCell::new(HashMap::new()),
            outer,
        }
    }
}

// 3. 执行上下文
#[derive(Debug, Clone)]
struct ExecutionContext {
    // 为简化，我们只用一个词法环境
    lexical_env: Rc<LexicalEnvironment>,
}

// 4. JS 引擎，持有执行栈
struct JsEngine {
    // 执行栈
    stack: Vec<ExecutionContext>,
}

impl JsEngine {
    // 创建一个新引擎，并初始化全局执行上下文
    fn new() -> Self {
        // 创建全局词法环境，它没有外部环境
        let global_env = Rc::new(LexicalEnvironment::new(None));
        // 创建全局执行上下文
        let global_context = ExecutionContext {
            lexical_env: global_env,
        };

        JsEngine {
            // 将全局上下文压入栈底
            stack: vec![global_context],
        }
    }

    // 获取当前活动的执行上下文
    fn current_context(&self) -> &ExecutionContext {
        self.stack
            .last()
            .expect("Execution stack should not be empty")
    }

    // 核心：变量解析（作用域链查找）
    fn resolve_variable(&self, name: &str) -> Option<JsValue> {
        // 从当前环境开始查找
        let mut current_env = Some(self.current_context().lexical_env.clone());

        while let Some(env) = current_env {
            // 检查当前环境的绑定中是否有该变量
            if let Some(value) = env.bindings.borrow().get(name) {
                return Some(value.clone());
            }
            // 如果没有，顺着 outer 链接向上层作用域查找
            current_env = env.outer.clone();
        }

        // 整个作用域链都找不到
        None
    }

    // 执行一个函数
    fn execute(&mut self, func: &FunctionBody, args: HashMap<String, JsValue>) {
        // 1. 为函数调用创建一个新的词法环境
        //    这个新环境的外部链接是函数被创建时捕获的环境！这就是闭包的核心！
        let new_env = Rc::new(LexicalEnvironment::new(Some(func.outer_env.clone())));

        // 将参数放入新环境
        for (name, value) in args {
            new_env.bindings.borrow_mut().insert(name, value);
        }

        // 2. 创建新的执行上下文
        let new_context = ExecutionContext {
            lexical_env: new_env,
        };

        // 3. 压入执行栈
        println!("==> Stack Push: Entering function");
        self.stack.push(new_context);

        // 4. 执行函数体（用 Rust 闭包模拟）
        (func.body)(self);

        // 5. 弹出执行栈
        self.stack.pop();
        println!("<== Stack Pop: Exiting function");
    }
}

// main.rs (续)
fn main() {
    // 1. 初始化引擎，创建全局上下文
    let mut engine = JsEngine::new();

    // 2. 在全局环境中定义变量和函数
    let global_env = engine.current_context().lexical_env.clone();
    global_env
        .bindings
        .borrow_mut()
        .insert("y".to_string(), JsValue::Number(20.0));

    // 3. 定义 outerFunc
    //    它在全局环境中被创建，所以它捕获的 outer_env 是全局环境
    let outer_func_body = FunctionBody {
        outer_env: global_env.clone(),
        body: Rc::new(Box::new(|engine: &mut JsEngine| {
            // --- 这是 outerFunc 的函数体 ---
            println!("Inside outerFunc");

            // a. 在 outerFunc 的环境中定义变量 z
            let outer_env = engine.current_context().lexical_env.clone();
            outer_env
                .bindings
                .borrow_mut()
                .insert("z".to_string(), JsValue::Number(30.0));

            // b. 在 outerFunc 的环境中定义 innerFunc
            //    它捕获的 outer_env 是 outerFunc 的环境
            let inner_func_body = FunctionBody {
                outer_env: outer_env.clone(),
                body: Rc::new(Box::new(|engine: &mut JsEngine| {
                    // --- 这是 innerFunc 的函数体 ---
                    println!("Inside innerFunc");
                    // c. 解析变量 (作用域链查找)
                    let x = engine.resolve_variable("x").unwrap();
                    let y = engine.resolve_variable("y").unwrap();
                    let z = engine.resolve_variable("z").unwrap();

                    if let (
                        JsValue::Number(x_val),
                        JsValue::Number(y_val),
                        JsValue::Number(z_val),
                    ) = (x, y, z)
                    {
                        println!("Result: {}", x_val + y_val + z_val);
                    }
                })),
            };

            // d. 执行 innerFunc
            engine.execute(&inner_func_body, HashMap::new());
        })),
    };
    let outer_func = JsValue::Function(outer_func_body);
    global_env
        .bindings
        .borrow_mut()
        .insert("outerFunc".to_string(), outer_func);

    // 4. 从全局环境中获取并执行 outerFunc
    if let Some(JsValue::Function(func_to_run)) = engine.resolve_variable("outerFunc") {
        let mut args = HashMap::new();
        args.insert("x".to_string(), JsValue::Number(10.0));
        engine.execute(&func_to_run, args);
    }
}
