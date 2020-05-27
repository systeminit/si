var myArray = [
  { key: 'myKey01', value: 'value of myKey01' },
  { key: 'myKey02', value: 'value of myKey02' },
  { key: 'myKey03', value: 'value of myKey03' },
]

// var myMap = arr.reduce(function(map, obj) {
//     map[obj.key] = obj.val;
//     return map;
// }, {});

// arr.reduce(callback( accumulator, currentValue[, index[, array]] )[, initialValue])

var res = myArray.reduce((array,arrayItem) => (array[arrayItem.key]=arrayItem.value, array),{});

console.log(res)