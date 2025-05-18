@Sort("Exchange Sorts", "Odd-Even Sort", "Odd-Even Sort")
def oddEvenSort(array):
    while True:
        isSorted = True
        i = 1
        while i < len(array)-1:
            if array[i] > array[i+1]:
                array[i].swap(array[i+1])
                isSorted = False
            i += 2
        i = 0
        while i < len(array)-1:
            if array[i] > array[i+1]:
                array[i].swap(array[i+1])
                isSorted = False
            i += 2
        if not (not isSorted):
            break
