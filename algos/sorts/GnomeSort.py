def gnomeSort(array, a, b):
    i = a+1
    while i < b:
        if array[i] >= array[i-1]:
            i += 1
        else:
            array[i].swap(array[i-1])
            if i > 1:
                i -= 1


@Sort("Exchange Sorts", "Gnome Sort", "Gnome Sort")
def gnomeSortRun(array):
    gnomeSort(array, 0, len(array))
