@Sort("Pancake Sorts", "Pancake Sort", "Pancake Sort")
def pancakeSort(array):
    i = len(array)-1
    while i >= 0:
        if not checkSorted(array, 0, i+1):
            index = findMaxIndex(array, 0, i+1)
            if index == 0:
                reverse(array, 0, i+1)
            elif index != i:
                reverse(array, 0, index+1)
                reverse(array, 0, i+1)
        i -= 1
