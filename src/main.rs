use uci::{gameloop, VERSION};

mod eval;
mod movegen;
pub mod search;
mod uci;

fn main() {
    println!(
        "   
         :.....:-==:         
      ::.          .*@#=      
    :.               +@@@+    
  ::         +@@+     @@@@@:  
 :.          =%%=     @@@@@@- 
 :                   *@@@@@@@.
-                  :#@@@@@@@@=
-           :==++#@@@@@@@@@@@*
-        .*@@@@@@@@@@@@@@@@@@=
.:      -@@@@@@@@@@@@@@@@@@@@.
 :.     @@@@@+..+@@@@@@@@@@@- 
  ::    @@@@@=  =@@@@@@@@@@-  
    :.  =@@@@@@@@@@@@@@@@+    
       :.-%@@@@@@@@@@+     
"
    );
    println!("ShenYu v{VERSION} by Aaron Li");
    gameloop();
}
