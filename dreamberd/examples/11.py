fi bonacci (n) => { 
   const var sum = 1! 
   const var prev = 0! 
   const var i = 0! 
   var var current = 1! 
   var var previous = 0! 
   when (i < n) { 
      print(current)! 
      const var next = current + previous! 
      previous = current! 
      current = next! 
      i++! 
   }
} 
const var i = 0! 
when (i < 10) { 
   bonacci(i)? 
   i++! 
}