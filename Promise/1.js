const PENDING = "pending";
const FULFILLED = "fulfilled";
const REJECTED = "rejected";

function runMicroTask(callback, data, resolve, reject) {
  setTimeout(() => {
    try {
      const res = callback(data);
      resolve(res);
    } catch (e) {
      reject(e);
    }
  }, 0);
}

class MyPromise {
  constructor(executor) {
    this.status = PENDING;
    this.result = null;
    this.onFulfilledCallbacks = []; // [{ callback, resolve, reject }, ...]
    this.onRejectedCallbacks = []; // [{ callback, resolve, reject }, ...]
    executor(this.resolve.bind(this), this.reject.bind(this));
  }
  then(onFulfilled, onRejected) {
    return new MyPromise((resolve, reject) => {
      if (this.status === PENDING) {
        this.onFulfilledCallbacks.push({
          callback: onFulfilled,
          resolve,
          reject,
        });
        this.onRejectedCallbacks.push({
          callback: onRejected,
          resolve,
          reject,
        });
      } else if (this.status === FULFILLED) {
        runMicroTask(onFulfilled, this.result, resolve, reject);
      } else if (this.status === REJECTED) {
        runMicroTask(onRejected, this.result, resolve, reject);
      }
    });
  }
  resolve(data) {
    // console.log(data);
    this.changeStatus(FULFILLED, data);
    // console.log("this.result: ", this.result);

    this.handleCallbacks(FULFILLED);
  }
  reject(data) {
    // console.log(data);

    this.changeStatus(REJECTED, data);
    // console.log("this.result: ", this.result);
    this.handleCallbacks(REJECTED);
  }
  handleCallbacks(status) {
    let callbacks = [];
    status === FULFILLED
      ? (callbacks = this.onFulfilledCallbacks)
      : (callbacks = this.onRejectedCallbacks);

    while (callbacks.length) {
      const { callback, resolve, reject } = callbacks[0];
      callbacks.shift();
      runMicroTask(callback, this.result, resolve, reject);
    }
  }

  changeStatus(status, data) {
    if (this.status !== PENDING) {
      return;
    }
    this.status = status;
    this.result = data;
  }
}

const pm = new MyPromise((resolve, reject) => {
  const num = Math.random();
  setTimeout(() => {
    if (num > 0.5) {
      resolve("Yes, indeed");
    } else {
      reject("Sorry...");
    }
  }, 1000);
});

pm.then(
  (data) => {
    console.log(data);
    return "happy promise2";
  },
  (data) => {
    console.log(data);
    throw "sad promise2";
  }
).then(
  (data) => {
    console.log(data);
  },
  (data) => {
    console.log(data);
  }
);
