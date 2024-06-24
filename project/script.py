# strings = [
#     "if",
#     "else",
#     "procedure",
#     "is",
#     "global",
#     "variable",
#     "begin",
#     "then",
#     "end",
#     "program"
# ]

# for word in strings:
#     print('("' + word + '", Token::new(tokenTypeEnum::' + word.upper() + ', "' + word + '".to_string())),')



def scanThrough(testStr):
    # scope = 0
    i = 1

    print("Length: " + str(len(testStr)))

    statement = []

    while (i < len(testStr)):
        currentLetter = testStr[i]
        
        print("Character: " + currentLetter)
        if(currentLetter == '('):
            # scope = scope + 1
            print("Found sub statement")
            newStr = testStr[i:]
            scanned = scanThrough(newStr)
            statement.append(scanned)
            diff = len(testStr) - len(scanned)
            i = i + diff
        elif (currentLetter == ')'):
            print("Closing bracket found")
            i = len(testStr) + 1
        else:
            statement.append(currentLetter)
            i = i + 1
        
    return statement
        # i = i + 1


testStr = "(a + (b + c))"

testList = scanThrough(testStr)


def print_tiered_list(lst, level=0):
    indent = "  " * level  # Create indentation based on the level
    for item in lst:
        if isinstance(item, list):
            print_tiered_list(item, level + 1)  # Recursively print nested lists
        else:
            print(indent + str(item))  # Print the item with indentation

# Example usage
testStr = "(a + (b + c))"

testList = scanThrough(testStr)

# Remove spaces from testList
def remove_spaces(lst):
    result = []
    for item in lst:
        if isinstance(item, list):
            result.append(remove_spaces(item))
        elif item != ' ':
            result.append(item)
    return result

testList = remove_spaces(testList)

# Print the tiered list
print_tiered_list(testList)
