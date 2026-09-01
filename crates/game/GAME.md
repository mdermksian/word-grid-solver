# Word Grid Game

The game is generally as follows:
1. Users randomize dice and start timer
2. Users may create words at least the size of the minimum word length by traversing the grid in any cardinal or ordinal direction without doubling back or reusing a die
3. Any players with the same word must remove that word from their list
4. The remaining words are added up and scored
5. 1-4 are repeated for as many rounds as the players like, and the final score is the sum of the rounds played.

## In-game mechanics
1. When a player finds a word, the word's pattern should be overlaid on the board temporarily
2. Randomizing the dice at the start of the game should be fun and satisfying
3. Music is timed to the rule-set, culminating in the completed time

## Configuration

The following features should be configurable:
1. Number of cubes
2. Minimum word length
3. Time limit
4. Scoring system
5. Cube set (non-standard sets)
6. Dictionary (?)

## Cube Sets
There may be an option to use different cubes in the future, but for now we should start with the following cube sets:

### Standard (New)
1. A E A N E G
2. A H S P C O
3. A S P F F K
4. O B J O A B
5. I O T M U C
6. R Y V D E L
7. L R E I X D
8. E I U N E S
9. W N G E E H
10. L N H N R Z
11. T S T I Y D
12. O W T O A T
13. E R T T Y L
14. T O E S S I
15. T E R W H V
16. N U I H M Qu

### Standard (Old)
1. A A C I O T
2. A B I L T Y
3. A B J M O Qu
4. A C D E M P
5. A C E L R S
6. A D E N V Z
7. A H M O R S
8. B I F O R X
9. D E N O S W
10. D K N O T U
11. E E F H I Y
12. E G K L U Y
13. E G I N T V
14. E H I N P S
15. E L P S T U
16. G I L R U W


### Big
1. A A A F R S
2. A A E E E E
3. A A F I R S
4. A D E N N N
5. A E E E E M
6. A E E G M U
7. A E G M N N
8. A F I R S Y
9. B J K Q X Z
10. C C E N S T
11. C E I I L T
12. C E I L P T
13. C E I P S T
14. D D H N O T
15. D H H L O R
16. D H L N O R
17. D H L N O R
18. E I I I T T
19. E M O T T T
20. E N S S S U
21. F I P R S Y
22. G O R R V W
23. I P R R R Y
24. N O O T U W
25. O O O T T U

## Scoring System
| Number of Letters | 3 | 4 | 5 | 6 | 7 | 8+ |
| --- | --- | --- | --- | --- | --- | --- |
| Points | 1 | 1 | 2 | 3 | 5 | 11 |

Discard the columns below the minimum number of letters for larger games.

## Game modes

### Normal
Normal game mode has the following configuration:
1. Grid Size: 4x4
2. Minimum word length: 3min
3. Time Limit: 3 min
4. Cube Set: Standard (New) or Standard (Old)
5. Scoring system: Standard

### Big
Big game mode has the following configuration:
1. Grid Size: 5x5
2. Minimum word length: 4
3. Time Limit: 3 min
4. Cube Set: Big
5. Scoring system: Standard

### Custom
1. Grid Size: User choice, or custom defined
2. Minimumn word length: User choice >=3
3. Time Limit: User choice
4. Cube Set: User choice matching grid size
5. Scoring system: Standard or user choice

### Endless
1. Normal
2. Big
3. Custom

In endless mode, the full word list is hidden from the user and as they find words, they are added to a list. The goal is to find as many words as possible.