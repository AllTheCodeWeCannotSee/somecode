// main.rs (可运行的最终版)
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// --- 核心数据结构 ---

#[derive(Clone, Debug)]
enum JsValue {
    Undefined,
    String(String),
    Object(Rc<RefCell<HashMap<String, JsValue>>>),
    Function(FunctionBody),
}

#[derive(Clone)]
struct FunctionBody {
    outer_env: Rc<LexicalEnvironment>,
    body: Rc<Box<dyn Fn(&mut JsEngine)>>,
}

// NEW: 代表表达式求值的结果
#[derive(Clone)]
enum EvaluationResult {
    Value(JsValue), // 只是一个普通的值, e.g. greet
    Reference {
        // 一个带有基对象的引用, e.g. person.greet
        base: JsValue,
        property_name: String,
    },
}

#[derive(Debug, Clone)]
struct LexicalEnvironment {
    bindings: RefCell<HashMap<String, JsValue>>,
    outer: Option<Rc<LexicalEnvironment>>,
}
#[derive(Debug, Clone)]
struct ExecutionContext {
    lexical_env: Rc<LexicalEnvironment>,
    this_binding: JsValue,
}

struct JsEngine {
    stack: Vec<ExecutionContext>,
    global_object: JsValue,
}

// --- 辅助函数 ---
fn new_object(props: Option<HashMap<String, JsValue>>) -> JsValue {
    JsValue::Object(Rc::new(RefCell::new(props.unwrap_or_default())))
}

fn get_property(obj: &JsValue, prop_name: &str) -> JsValue {
    if let JsValue::Object(o) = obj {
        o.borrow()
            .get(prop_name)
            .cloned()
            .unwrap_or(JsValue::Undefined)
    } else {
        JsValue::Undefined
    }
}

// --- 引擎实现 ---
impl JsEngine {
    fn new() -> Self {
        let global_env = Rc::new(LexicalEnvironment::new(None));
        let global_object = new_object(None);
        let global_context = ExecutionContext {
            lexical_env: global_env,
            this_binding: global_object.clone(),
        };
        Self {
            stack: vec![global_context],
            global_object,
        }
    }

    fn resolve_this(&self) -> JsValue {
        self.stack.last().unwrap().this_binding.clone()
    }
    fn resolve_variable(&self, name: &str) -> Option<JsValue> {
        // ... (代码与上一版相同)
        let mut current_env = Some(self.stack.last().unwrap().lexical_env.clone());
        while let Some(env) = current_env {
            if let Some(value) = env.bindings.borrow().get(name) {
                return Some(value.clone());
            }
            current_env = env.outer.clone();
        }
        None
    }

    // NEW: 更智能的 invoke 方法，负责决策 this
    fn invoke(&mut self, eval_result: EvaluationResult, args: HashMap<String, JsValue>) {
        // --- 引擎自动决策 this 的核心逻辑 ---
        let (callee, this_arg) = match eval_result {
            // 情况 A: 如果求值结果是一个引用 (e.g., person.greet)
            EvaluationResult::Reference {
                base,
                property_name,
            } => {
                println!("[Engine]: Detected a Method Call (via Reference).");
                let method = get_property(&base, &property_name);
                // 将引用的 base 作为 this
                (method, base)
            }
            // 情况 B: 如果求值结果只是一个普通值 (e.g., greet)
            EvaluationResult::Value(value) => {
                println!("[Engine]: Detected a Standalone Function Call.");
                // this 使用默认绑定，即全局对象
                (value, self.global_object.clone())
            }
        };

        // --- 决策结束，开始执行 ---
        self.execute(callee, this_arg, args);
    }

    // execute 现在更像一个内部方法，负责纯粹的执行流程
    fn execute(&mut self, callee: JsValue, this_arg: JsValue, args: HashMap<String, JsValue>) {
        let func_body = match callee {
            JsValue::Function(body) => body,
            _ => {
                println!("[Error]: Attempted to call a non-function value.");
                return;
            }
        };

        let new_env = Rc::new(LexicalEnvironment::new(Some(func_body.outer_env.clone())));
        for (name, value) in args {
            new_env.bindings.borrow_mut().insert(name, value);
        }
        let new_context = ExecutionContext {
            lexical_env: new_env,
            this_binding: this_arg,
        };

        println!("==> Stack Push: Entering function. `this` is bound.");
        self.stack.push(new_context);
        (func_body.body)(self);
        self.stack.pop();
        println!("<== Stack Pop: Exiting function.");
    }
}

// --- 为了方便 Debug 打印 ---
impl std::fmt::Debug for FunctionBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Function]")
    }
}
impl LexicalEnvironment {
    fn new(outer: Option<Rc<LexicalEnvironment>>) -> Self {
        Self {
            bindings: RefCell::new(HashMap::new()),
            outer,
        }
    }
}

// ---------------------------------- main --------------------------------- //

fn main() {
    // 1. 初始化阶段 (与之前相同)
    let mut engine = JsEngine::new();
    let global_env = engine.stack[0].lexical_env.clone();
    if let JsValue::Object(g_obj) = &engine.global_object {
        g_obj
            .borrow_mut()
            .insert("name".to_string(), JsValue::String("Global".to_string()));
    }
    let greet_func_body = FunctionBody {
        outer_env: global_env.clone(),
        body: Rc::new(Box::new(|engine: &mut JsEngine| {
            let this_val = engine.resolve_this();
            if let JsValue::Object(this_obj) = this_val {
                if let Some(JsValue::String(name)) = this_obj.borrow().get("name") {
                    println!("Hello, {}", name);
                }
            }
        })),
    };
    let greet_func = JsValue::Function(greet_func_body);
    global_env
        .bindings
        .borrow_mut()
        .insert("greet".to_string(), greet_func.clone());
    let mut person_props = HashMap::new();
    person_props.insert("name".to_string(), JsValue::String("Alice".to_string()));
    person_props.insert("greet".to_string(), greet_func.clone());
    let person_obj = new_object(Some(person_props));

    println!("--- 第一次调用: 模拟 `greet()` ---");
    // 2. 模拟对 `greet` 表达式的求值
    println!("Simulating evaluation of: `greet`");
    let eval_result1 = EvaluationResult::Value(engine.resolve_variable("greet").unwrap());
    // 将求值结果交给引擎去调用
    engine.invoke(eval_result1, HashMap::new());

    println!("\n--- 第二次调用: 模拟 `person.greet()` ---");
    // 3. 模拟对 `person.greet` 表达式的求值
    println!("Simulating evaluation of: `person.greet`");
    let eval_result2 = EvaluationResult::Reference {
        base: person_obj.clone(),
        property_name: "greet".to_string(),
    };
    // 将求值结果交给引擎去调用
    engine.invoke(eval_result2, HashMap::new());
}
