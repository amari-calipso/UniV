def countingSort(array, a, b):
    max_ = findMax(array, a, b)
    counts = sortingVisualizer.createValueArray(max_+1)
    output = sortingVisualizer.createValueArray(b-a)
    sortingVisualizer.setNonOrigAux([counts, output])
    arrayCopy(array, a, output, 0, b-a)
    i = a
    while i < b:
        counts[array[i].readInt()] += 1
        i += 1
    i = 1
    while i < max_+1:
        counts[i] += counts[i-1]
        i += 1
    i = b-1
    while i >= 0:
        output[counts[array[i].readInt()].readInt()-1].write(array[i])
        counts[array[i].getInt()] -= 1
        i -= 1
    reverseArrayCopy(output, 0, array, a, b-a)


@Sort("Distribution Sorts", "Counting Sort", "Counting Sort")
def countingSortRun(array):
    countingSort(array, 0, len(array))
