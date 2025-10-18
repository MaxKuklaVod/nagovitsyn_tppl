const var i = 0: Int! 
when (i < 20) { 
   when (i % 3 = 0 && i % 5 = 0) print("FizzBuzz")? 
   else when (i % 3 = 0) print("Fizz")? 
   else when (i % 5 = 0) print("Buzz")? 
   else print(i)? 
   i++! 
}