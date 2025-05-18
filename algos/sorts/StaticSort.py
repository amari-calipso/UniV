def staticSort(array, a, b):
    min_: int
    max_: int
    min_, max_ = findMinMax(array, a, b)
    auxLen = b-a
    offset = sortingVisualizer.createValueArray(auxLen+1)
    count = sortingVisualizer.createValueArray(auxLen+1)
    sortingVisualizer.setNonOrigAux([offset, count])
    CONST = auxLen/(max_-min_+1)
    i = a
    while i < b:
        count[int((array[i].readInt()-min_)*CONST)] += 1
        i += 1
    offset[0].write(a)
    i = 1
    while i < auxLen:
        offset[i].write(count[i-1]+offset[i-1])
        i += 1
    for v in range(auxLen):
        while count[v] > 0:
            origin = offset[v].readInt()
            from_ = origin
            num = array[from_].copy()
            array[from_].write(-1)
            while True:
                dig = int((num.readInt()-min_)*CONST)
                to = offset[dig].readInt()
                offset[dig] += 1
                count[dig] -= 1
                temp = array[to].copy()
                array[to].write(num)
                num = temp.copy()
                from_ = to
                if not (from_ != origin):
                    break
    UniV_removeAux(count)
    for i in range(auxLen):
        begin = offset[i-1].readInt()if i > 0 else a
        end = offset[i].readInt()
        if end-begin > 1:
            if end-begin > 16:
                MaxHeapSort().sort(array, begin, end)
            else:
                uncheckedInsertionSort(array, begin, end)


@Sort("Distribution Sorts", "Static Sort [Utils.Iterables.fastSort]", "Static Sort")
def staticSortRun(array):
    staticSort(array, 0, len(array))
