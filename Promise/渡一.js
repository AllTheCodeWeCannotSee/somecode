const PENDING = "pending";
const FULFILLED = "fulfilled";
const REJECTED = "rejected";

/**
 * 以微任务方式执行回调
 * @param {Function} callback
 */
function runMicroTask(callback) {
  // 优先使用 Node.js 的 process.nextTick
  if (process && process.nextTick) {
    process.nextTick(callback);
  }
  // 其次尝试浏览器环境的 MutationObserver
  else if (typeof MutationObserver === "function") {
    const obs = new MutationObserver(callback);
    const p = document.createElement("p");
    obs.observe(p, {
      childList: true,
    });
    p.innerHTML = "1";
  }
  // 降级为宏任务 setTimeout
  else {
    setTimeout(callback, 0);
  }
}

/**
 * 判断一个对象是否是 Promise (或 thenable)
 * @param {any} obj
 * @returns {boolean}
 */
function isPromise(obj) {
  return !!(obj && typeof obj === "object" && typeof obj.then === "function");
}

class MyPromise {
  /**
   * @param {Function} executor 任务执行器 (resolve, reject) => {}
   */
  constructor(executor) {
    this._state = PENDING; // Promise 状态
    this._value = undefined; // 终值或拒因
    this._handlers = []; // 处理器队列，存储 .then 的回调

    // 捕获 executor 执行中的同步错误
    try {
      executor(this._resolve.bind(this), this._reject.bind(this));
    } catch (error) {
      this._reject(error);
    }
  }

  /**
   * 更改状态并执行处理器队列
   * @param {string} newState 新状态
   * @param {any} value 终值或拒因
   * @private
   */
  _changeState(newState, value) {
    // 状态一旦改变就不能再变
    if (this._state !== PENDING) {
      return;
    }
    this._state = newState;
    this._value = value;
    // 状态改变后，异步执行所有相关的处理器
    this._runHandlers();
  }

  /**
   * 标记为成功状态
   * @param {any} data 终值
   * @private
   */
  _resolve(data) {
    this._changeState(FULFILLED, data);
  }

  /**
   * 标记为失败状态
   * @param {any} reason 拒因
   * @private
   */
  _reject(reason) {
    this._changeState(REJECTED, reason);
  }

  /**
   * 将 then 的回调函数包装成处理器推入队列
   * @param {Function} executor then传入的回调函数 (onFullfilled或onRejected)
   * @param {string} state 此处理器对应的状态
   * @param {Function} resolve then返回的新Promise的resolve函数
   * @param {Function} reject then返回的新Promise的reject函数
   * @private
   */
  _pushHandler(executor, state, resolve, reject) {
    this._handlers.push({
      executor,
      state,
      resolve,
      reject,
    });
  }

  /**
   * 执行处理器队列
   * @private
   */
  _runHandlers() {
    // 只有在非 PENDING 状态下才执行
    if (this._state === PENDING) {
      return;
    }
    // 依次执行队列中的所有处理器
    while (this._handlers.length) {
      const handler = this._handlers.shift();
      this._runOneHandler(handler);
    }
  }

  /**
   * 执行单个处理器
   * @param {object} handler 处理器对象
   * @private
   */
  _runOneHandler({ executor, state, resolve, reject }) {
    // 使用微任务来确保异步执行
    runMicroTask(() => {
      // 如果当前Promise状态与处理器要处理的状态不匹配，则不执行
      if (this._state !== state) {
        return;
      }

      // 如果 then 传入的不是一个函数 (e.g., .then(null, ...))，则状态和值直接穿透
      if (typeof executor !== "function") {
        this._state === FULFILLED ? resolve(this._value) : reject(this._value);
        return;
      }

      // 执行回调，并处理其返回值
      try {
        const result = executor(this._value);
        // 根据 Promise A+ 规范，处理回调的返回值
        if (isPromise(result)) {
          // 如果返回值是一个 Promise，则等待它完成
          result.then(resolve, reject);
        } else {
          // 如果是普通值，则直接 resolve 新的 Promise
          resolve(result);
        }
      } catch (error) {
        // 如果回调执行出错，则 reject 新的 Promise
        reject(error);
      }
    });
  }

  /**
   * Promise 的核心方法，用于注册成功和失败的回调
   * @param {Function} onFullfilled 成功回调
   * @param {Function} onRejected 失败回调
   * @returns {MyPromise} 返回一个新的 Promise，实现链式调用
   */
  then(onFullfilled, onRejected) {
    return new MyPromise((resolve, reject) => {
      this._pushHandler(onFullfilled, FULFILLED, resolve, reject);
      this._pushHandler(onRejected, REJECTED, resolve, reject);
      // 如果在调用 a.then(...) 时，a 的状态已经确定，则立即尝试执行处理器
      this._runHandlers();
    });
  }
}
