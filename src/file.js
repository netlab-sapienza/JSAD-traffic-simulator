const exp = require('constants')
const fs = require('fs')
let counter = 0

/*
let data = fs.readFileSync('4koutput.csv').toString().split('\n').map(row => row.split(',').map(el => el.trim()).map(elem => {
	console.log(elem.split(' '))
	let v = elem.split(' ')[1]
	if (v && !v.includes('ms') && v.includes('s')) {
		return parseFloat(v) * 1000
	}
	return parseFloat(v)

}).join(',')).join('\n')
.split('\n')
.map(row => {
	return row.split(',')
}).sort((r1,r2) => {
	//console.log(r1,r2)
	return parseFloat(r1[2])-parseFloat(r2[2])
}).map(row => {
	//console.log(row)
	row[3] = counter
	counter += 1
	return row.join(',')	
}).join('\n')
fs.writeFileSync('4k_output_parsed.csv', data)


let average_propor1 = 0
let average_propor2 = 0

let average1 = 0
let average2 = 0
*/


let avg_times = []
let average_propor1 = 0;
let average1 = 0
let total = 0

fs.readFileSync('times_fbfcfs.csv').toString().split('\n').map(row => row.split(',').map(elem => parseFloat(elem))).map(elem => {
        let [id, exp_time, real_time, order] = elem
        average_propor1 += (real_time/exp_time)
        average1 += real_time
        total += 1
})

console.log(`total: ${total}`)
console.log(`media fbfcfs: ${average1 / total}`)
console.log(`media propor fbfcfs: ${average_propor1 / total}`)
/*
fs.readFileSync('10k_fcfsfb_parsed.csv').toString().split('\n').map(row => row.split(',').map(elem => parseFloat(elem))).map(elem => {
        let [id, exp_time, real_time, order] = elem
        average_propor2 += (real_time/exp_time)
        average2 += real_time
})

console.log(`media fbfcfs: ${average1 / 10000}`)
console.log(`media fcfsfb: ${average2 / 10000}`)

console.log(`media propor fbfcfs: ${average_propor1 / 10000}`)
console.log(`media propor fcfsfb: ${average_propor2 / 10000}`)
*/
