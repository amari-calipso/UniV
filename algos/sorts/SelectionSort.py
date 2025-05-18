def selectionSort(array, a, b):
    i = a
    while i < b-1:
        lowest = i
        j = i+1
        while j < b:
            if array[j] < array[lowest]:
                lowest = j
            j += 1
        if lowest != i:
            array[i].swap(array[lowest])
        i += 1


def doubleSelectionSort(array, a, b):
    b -= 1
    while a <= b:
        lowest = a
        highest = a
        i = a+1
        while i <= b:
            if array[i] > array[highest]:
                highest = i
            elif array[i] < array[lowest]:
                lowest = i
            i += 1
        if highest == a:
            highest = lowest
        if a != lowest:
            array[a].swap(array[lowest])
        if b != highest:
            array[b].swap(array[highest])
        a += 1
        b -= 1


@Sort("Selection Sorts", "Selection Sort", "Selection Sort")
def selectionSortRun(array):
    selectionSort(array, 0, len(array))


@Sort("Selection Sorts", "Double Selection Sort", "Double Selection")
def doubleSelectionSortRun(array):
    doubleSelectionSort(array, 0, len(array))
