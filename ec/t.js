let y = 20;

function outerFunc(x) {
  let z = 30;

  function innerFunc() {
    // 将会查找 y, x, z
    console.log(x + y + z);
  }

  innerFunc();
}

outerFunc(10); // 期望输出 60
