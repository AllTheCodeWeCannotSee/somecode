

-----

### **学习笔记：Promise 链式调用的核心机制——“委托”的力量**

#### **引言：一个令人困惑的问题**

在手写或深入理解 `Promise.prototype.then` 方法时，我们都会遇到一个核心问题：为什么 `.then` 方法在返回 `promise2` 的同时，其内部的 `executor` 函数中充满了对 `promise1` (即 `this`) 状态的判断和处理？

这个设计的确有些违反直觉，但它正是实现 Promise 强大链式能力的关键。理解了这一点，就理解了 Promise 链的灵魂。

#### **核心原则：`.then()` 是一个“新承诺”的工厂**

`.then()` 方法的职责**不是**在原地等待或修改当前的 Promise (`promise1`)。它的核心职责是：

1.  接收一个“计划”（`onFulfilled` 回调）。
2.  **立即创建一个全新的 Promise (`promise2`)**，这个新 Promise 代表了“执行完计划后”的那个未来。
3.  安排一个“委托”任务：当 `promise1` 完成时，执行“计划” (`onFulfilled`)，并用计划的结果来决定 `promise2` 的最终命运。
4.  将这个全新的、处于 `pending` 状态的 `promise2` 返回。

#### **一个绝佳的比喻：接力赛**

  * **`promise1`**: 接力赛的**第一棒选手**。他正在奔跑。
  * **`promise2`**: **第二棒选手**。他现在站在等待区，还未起跑。
  * **`.then()` 方法**: **交接棒的过程**。这个过程本身就定义了第二棒选手该如何行动。

当你写 `const promise2 = promise1.then(onFulfilled)` 时，就相当于在规定：

> “`promise2`（第二棒选手）的比赛，完全**委托**于 `promise1`（第一棒选手）的完成情况。`promise2` 的 `executor` 要做的，就是定义好这个**委托和交接的规则**。”

-----

### **技术原理解析：为何必须如此设计？**

将 `promise1` 的处理逻辑放到 `promise2` 的 `executor` 中，是出于两个根本的技术原因：

**1. 为了获得对 `promise2` 的控制权**

根据 Promise 的设计，唯一能够改变一个 Promise 状态（从 `pending` 到 `fulfilled` 或 `rejected`）的地方，就是它自己的 `executor` 函数，因为只有 `executor` 才能接收到专属于自身的 `resolve` 和 `reject` 函数。

因此，任何想要决定 `promise2` 命运的逻辑，都**必须**被包裹在 `promise2` 的 `executor` 内部。

**2. 因为 `promise2` 的命运完全依赖于 `promise1`**

`promise2` 何时完成、成功与否，完全取决于：

  * `promise1` 何时完成。
  * `promise1` 完成后，你传给 `.then` 的回调函数 (`onFulfilled`) 的执行情况（是正常返回一个值，还是抛出错误）。

**将这两点结合起来，结论就显而易见了：**

> 我们必须在 `promise2` 的 `executor`（为了控制权）内部，编写一段逻辑来“监听” `promise1` 的状态（因为依赖性），并在 `promise1` 完成后，执行回调，最后用回调的结果来调用 `promise2` 的 `resolve` 或 `reject`。

所以，`promise2` 的 `executor` 并非在处理 `promise1` 的业务，而是在处理**如何从 `promise1` 过渡到 `promise2` 的“桥接逻辑”**。

-----

### **代码执行流程分解**

让我们回顾一下这个流程：

```javascript
// promise1.then(onFulfilled) 内部...

// 1. 立即创建并返回 promise2
return new MyPromise((resolve, reject) => { // 这里的 resolve, reject 属于 promise2

    // 2. 在 promise2 的 executor 中，定义如何处理 promise1 的结果
    const handler = () => {
        setTimeout(() => { // 保证异步
            try {
                // a. 执行用户提供的回调，并传入 promise1 的结果
                const x = onFulfilled(this.result); 
                // b. 用回调的结果 x，来决定 promise2 的命运
                resolve(x); // 调用的是 promise2 的 resolve
            } catch (error) {
                // c. 如果回调出错，则让 promise2 失败
                reject(error); // 调用的是 promise2 的 reject
            }
        }, 0);
    };

    // 3. 将这个“桥接逻辑”注册到 promise1 上
    if (this.status === PENDING) {
        this.onFulfilledCallbacks.push(handler);
    } else if (this.status === FULFILLED) {
        handler();
    }
});
```

这个结构清晰地展示了，`promise2` 的 `executor` 的全部工作，就是为 `promise1` 设置一个“监听器”（`handler`），这个监听器一旦被 `promise1` 触发，就会反过来控制 `promise2` 的状态。

### **结论**

将 `promise1` 的处理逻辑放到 `promise2` 的 `executor` 中，是实现 Promise 链式调用的核心设计。它构建了一个优雅的“委托”模型，让每一个 `.then()` 调用都能生成一个代表着序列中下一步的新 Promise，并将前后两个 Promise 的状态转换逻辑清晰地链接起来，从而构成了强大而灵活的异步数据流管道。