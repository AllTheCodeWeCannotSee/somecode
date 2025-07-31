const PENDING = "pending";
const FULFILLED = "fulfilled";
const REJECTED = "rejected";

class MyPromise {
  constructor(executor) {
    this.status = PENDING;
    this.result = undefined;
    this.onFulfilledCallbacks = [];
    this.onRejectedCallbacks = [];

    // 捕获 executor 中的同步错误
    try {
      executor(this.resolve.bind(this), this.reject.bind(this));
    } catch (error) {
      this.reject(error);
    }
  }

  resolve(data) {
    // resolve/reject 中也应该用 setTimeout 来模拟异步
    // 状态一旦改变就不可再变
    if (this.status === PENDING) {
      setTimeout(() => {
        this.status = FULFILLED;
        this.result = data;
        this.onFulfilledCallbacks.forEach((callback) => callback(this.result));
      }, 0);
    }
  }

  reject(data) {
    if (this.status === PENDING) {
      setTimeout(() => {
        this.status = REJECTED;
        this.result = data;
        this.onRejectedCallbacks.forEach((callback) => callback(this.result));
      }, 0);
    }
  }

  then(onFulfilled, onRejected) {
    onFulfilled =
      typeof onFulfilled === "function" ? onFulfilled : (value) => value;
    onRejected =
      typeof onRejected === "function"
        ? onRejected
        : (reason) => {
            throw reason;
          };

    const promise2 = new MyPromise((resolve, reject) => {
      if (this.status === FULFILLED) {
        setTimeout(() => {
          try {
            const x = onFulfilled(this.result);
            resolve(x);
          } catch (error) {
            reject(error);
          }
        }, 0);
      }

      if (this.status === REJECTED) {
        setTimeout(() => {
          try {
            const x = onRejected(this.result);
            resolve(x); // 注意：.then 的 onRejected 回调如果正常返回，新的 promise 是 fulfilled
          } catch (error) {
            reject(error);
          }
        }, 0);
      }

      if (this.status === PENDING) {
        this.onFulfilledCallbacks.push(() => {
          setTimeout(() => {
            try {
              const x = onFulfilled(this.result);
              resolve(x);
            } catch (error) {
              reject(error);
            }
          }, 0);
        });
        this.onRejectedCallbacks.push(() => {
          setTimeout(() => {
            try {
              const x = onRejected(this.result);
              resolve(x);
            } catch (error) {
              reject(error);
            }
          }, 0);
        });
      }
    });

    return promise2;
  }
}
