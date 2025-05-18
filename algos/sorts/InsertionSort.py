def insertionSort(array, a, b):
    i = a+1
    while i < b:
        key: Value
        idx: int
        key, idx = array[i].readNoMark()
        j = i-1
        while array[j] > key and j >= a:
            array[j+1].write(array[j].noMark())
            j -= 1
        array[j+1].writeRestoreIdx(key, idx)
        i += 1


@Sort("Insertion Sorts", "Insertion Sort", "Insertion Sort")
def runInsertionSort(array):
    insertionSort(array, 0, len(array))
