function mySetInterval(fn, delay) {
  let timer = null;
  function loop() {
    // 执行1次
    fn.apply(this);
    // setTimeout(loop, delay);
    timer = setTimeout(() => {
      loop();
    }, delay);
  }
  timer = setTimeout(() => {
    loop();
  }, delay);
  return timer;
}
