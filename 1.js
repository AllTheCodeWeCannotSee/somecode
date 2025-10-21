class Processor {
  maxTaskCount;
  curTaskCount;
  todoTaskArr;

  constructor(limit) {
    this.maxTaskCount = limit;
    this.curTaskCount = 0;
    this.todoTaskArr = [];
  }
}

class Task {
  id;

  constructor(id) {
    this.id = id;
  }
}

function beginWork() {
  console.log("---- begin ----");
  const processor = new Processor(3);

  const taskArray = [];
  for (let i = 0; i < 10; i++) {
    const nextTask = new Task(i);
    taskArray.push(nextTask);
  }

  processor.todoTaskArr = taskArray;
  runTask(processor);
}

function runTask(processor) {
  while (
    processor.curTaskCount < processor.maxTaskCount &&
    processor.todoTaskArr.length
  ) {
    const task = processor.todoTaskArr[0];
    processor.todoTaskArr.shift();
    processor.curTaskCount++;

    console.log("begin task: ", task.id);

    new Promise((resolve) => {
      const delay = Math.floor(Math.random() * 5000) + 500;
      setTimeout(() => {
        console.log("end task:", task.id);
        processor.curTaskCount--;
        runTask(processor);
      }, delay);
    });
  }
}

beginWork();
