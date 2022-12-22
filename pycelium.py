import random
import time
import os

GRID_WIDTH = 40
GRID_HEIGHT = 20
startingX = (GRID_WIDTH - 1) // 2
startingY = (GRID_HEIGHT - 1) // 2
foodIcon = '◍'
fungalHead = '●'
# Set the initial position of the fungal growth at the bottom center of the screen
x = startingX
y = startingY


# Set the position of the food source
food_x = random.randint(0, GRID_WIDTH - 1)
food_y = random.randint(0, GRID_HEIGHT - 1)

# Initialize the grid
grid = [[' ' for _ in range(GRID_WIDTH)] for _ in range(GRID_HEIGHT)]

def move_towards_food():
    """Move the fungal growth towards the food source."""
    global x, y
    # Calculate the distance between the fungal growth and the food source
    dx = food_x - x
    dy = food_y - y

    # Move the fungal growth in the direction of the food source
    if dx > 0:
        x += 1
    elif dx < 0:
        x -= 1
    if dy > 0:
        y += 1
    elif dy < 0:
        y -= 1

while True:
    # Draw the food source
    os.system('cls')
    print("Pycelium")
    
    # # Draw the grid
    # print("+" + "-"*GRID_WIDTH + "+")
    # for row in grid:
    #     print("|" + " ".join(row) + "|")
    # print("+" + "-"*GRID_WIDTH + "+")
    
    grid[food_y][food_x] = foodIcon

    # Draw the fungal growth
    grid[y][x] = fungalHead

    # Draw the mycelium trails
    for i in range(GRID_HEIGHT):
        for j in range(GRID_WIDTH):
            if grid[i][j] == ' ':
                grid[i][j] = ' '
    print("+" + "-"*80 + "+")


    
    # Print the grid
    for row in grid:
        print(' '.join(row))
    print("+" + "-"*80 + "+")
    print(" ")
    print("Legend:")
    print(fungalHead, "- Fungal growth")
    print(foodIcon, "- Food source")
    # Pause the animation
    time.sleep(0.1)

    # Move the fungal growth towards the food source
    move_towards_food()
    
        # Check if the fungal growth has reached the food source
    if x == food_x and y == food_y:
        #remove the food from the board
        grid[food_y][food_x] = ' '
        # Spawn a new food source
        food_x = random.randint(0, GRID_WIDTH - 1)
        food_y = random.randint(0, GRID_HEIGHT - 1)
        # Spawn a new fungal growth as a branch of the original, closer to the food source
        x = startingX
        y = startingY
        # Move the new fungal growth towards the food source            
        move_towards_food()
    