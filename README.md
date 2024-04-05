# Tutto Simulation

This is a crate to analyze strategies in the game Tutto.

## The Game

The following section explains the rules to the game.
Note that these aren't the official rules, but rules as I personally play the game with friends.
But the rules should be close enough to make no big difference in strategy. Though I haven't tested this.

### Dice

There are 6 dice in the game.
After the dice are rolled the player needs to put a side at least one dice.
Single 1s count 100 points and single 5s count 50.
Additionally triplets count as hundred time their value. For example a 4 triplet is worth 400 point. Except 1 triplets which count 1000 points.
After puting aside the chosen dice the player can decide if they want to roll again or take the points. If the player cannot take any dice they loose all their points.

### Cards

There are 7 different cards:
Bonus 200  5    
Bonus 300  5    
Bonus 400  5    
Bonus 500  5    
Bonus 600  5    
Double  5      
FireWork  5    
Flush     5      
Clover  1      
Stop  10       
PlusMinus  5   

For all Bonus cards if the player achieves a tutto the bonus points are counted to the total.
The Double card doubles the points collected while the card was open.
During a FireWork card the player can roll their dice until they can't put away any dice anymore. On tutto no new card is drawn.
The Flush card works different than any other card. The goal is to get one of each number. This can be achieved in multiple rolls where in each roll at least one dice must be put aside.
For the Clover card normal dice rules apply. The points are not couted. If the player achieves a tutto twice they win the game.
A Stop card prevents the player from playing and if they had points from previous cards these are lost.
The PlusMinus card allows the player to steal a thousand points from the highest scoring player or players if there are more than one at the top. If the player themself is the highest scoring player they are skipped. The player isn't allowed to draw a new card after achieving a tutto.
