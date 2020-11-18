#!/usr/bin/env node

const fs = require('fs');

function processBgmFile(filename) {
  let data = fs.readFileSync("original/" + filename);
  let lines = data.toString().split("\n");
  let result = new Array();

  const track_number_re = /^\[(\d+)\]/;
  const offsets_re = /^position = \"(.*)\"/;

  lines.forEach((line) => {
    if ((captures = line.match(track_number_re)) !== null) {
      result.push("[[tracks]]");
      result.push("track_number = " + Number(captures[1]));
    }
    else if ((captures = line.match(offsets_re)) !== null) {
      result.push("position = [" + captures[1] + "]");
    }
    else {
      result.push(line);
    }
  });

  fs.writeFileSync(filename, result.join("\n"));
}

fs.readdirSync("original/").forEach((file) => {
  if (file.endsWith(".bgm")) {
    processBgmFile(file);
  }
});
