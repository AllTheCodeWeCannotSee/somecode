var name = "Global"; // 实际是 window.name = 'Global'

function greet() {
  // this 的值取决于调用方式
  console.log("Hello, " + this.name);
}

let person = {
  name: "Alice",
  greet: greet, // 将同一个 greet 函数赋给 person 的一个属性
};

// 调用方式 1: 默认绑定
// 期望 'this' 指向全局对象, 输出 "Hello, Global"
greet();

// 调用方式 2: 隐式绑定
// 期望 'this' 指向 person 对象, 输出 "Hello, Alice"
person.greet();
