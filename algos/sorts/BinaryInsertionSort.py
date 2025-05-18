def binaryInsertionSort(array, a, b):
    i = a+1
    while i < b:
        if array[i] < array[i-1]:
            insertToLeft(array, i, lrBinarySearch(
                array, a, i, array[i], False))
        i += 1


@Sort("Insertion Sorts", "Binary Insertion Sort", "Binary Insertion")
def binaryInsertionSortRun(array):
    binaryInsertionSort(array, 0, len(array))
