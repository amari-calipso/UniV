def uncheckedInsertionSort(array, a, b):
    i = a+1
    while i < b:
        if array[i] < array[a]:
            array[i].swap(array[a])
        key: Value
        idx: int
        key, idx = array[i].readNoMark()
        j = i-1
        while array[j] > key:
            array[j+1].write(array[j].noMark())
            j -= 1
        array[j+1].writeRestoreIdx(key, idx)
        i += 1


@Sort("Insertion Sorts", "Unstable Insertion Sort", "Unstable Insertion")
def uncheckedInsertionSortRun(array):
    uncheckedInsertionSort(array, 0, len(array))
