class Queue {
  // private buffer_capacity: number
  constructor(capacity) {
      this.start = 0;
      this.count = 0;
      // bufferサイズの制限(上限ではない)
      this.capacity = 512;
      if (capacity) {
          this.buffer = new Array(capacity);
          this.capacity = capacity;
      }
      else {
          this.buffer = new Array();
      }
  }
  at(index) {
      if (index < 0) {
          let t = index + this.count;
          if (t < 0) {
              return undefined;
          }
          else {
              return this.buffer[index + this.start + this.count];
          }
      }
      else {
          return this.buffer[index + this.start];
      }
  }
  first() {
      return this.buffer[this.start];
  }
  last() {
      return this.buffer[this.count + this.start];
  }
  /* 操作系メソッド */
  change_at(idx, value) {
      if (idx < 0) {
          let t = idx + this.count;
          if (t >= 0) {
              this.buffer[idx + this.start + this.count] = value;
          }
      }
      else {
          this.buffer[idx + this.start] = value;
      }
  }
  change_at_by(idx, f) {
      let index;
      if (idx < 0) {
          index = idx + this.count;
          if (index < 0) {
              index = this.start;
          }
          else {
              index += this.start;
          }
      }
      else {
          index = this.start + idx;
      }
      let v = this.buffer[index];
      if (v) {
          this.buffer[index] = f(v);
      }
  }
  change_last(value) {
      this.buffer[this.start + this.count] = value;
  }
  change_last_by(f) {
      let v = this.buffer[this.start + this.count];
      if (v) {
          this.buffer[this.start + this.count] = f(v);
      }
  }
  /**
   * @description
   * @param {T} value
   */
  enqueue(value) {
      if (this.start + this.count + 1 > this.capacity) {
          this.shrink_to_fit();
      }
      this.buffer[this.start + this.count++] = value;
  }
  /**
   * @description
   * @returns {T|undefined}
   */
  dequeue() {
      if (this.count) {
          let item = this.buffer[this.start];
          this.buffer[this.start++] = undefined;
          this.count--;
          return item;
      }
      else {
          return undefined;
      }
  }
  /**
   * @description
   * @param {Array<T>}values
   */
  append(values) {
      let l = values.length;
      // データをシフト
      if (this.start + this.count + l > this.capacity) {
          this.shrink_to_fit();
      }
      // Queueに追加
      let offset = this.start + this.count;
      for (let i = 0; i < l; i++) {
          this.buffer[i + offset] = values[i];
      }
      this.count += l;
  }
  /**
   * @description
   */
  shrink_to_fit() {
      if (this.start) {
          let buf = this.buffer;
          let n = this.count < this.buffer.length ? this.count : this.buffer.length;
          let offset = this.start;
          for (let i = 0; i < n; i++) {
              buf[i] = buf[i + offset];
          }
          this.buffer.length = n > this.capacity ? n : this.capacity;
          this.start = 0;
      }
  }
  /**
   * @description Clears the Queue, removing all values.
   */
  clear() {
      this.start = 0;
      this.count = 0;
      for (let i = this.start; i < this.count; i++) {
          this.buffer[i] = undefined;
      }
      this.buffer.length = this.capacity;
  }
  get length() {
      return this.count;
  }
}
