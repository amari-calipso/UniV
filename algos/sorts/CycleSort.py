def cycleSort(array, a, b):
    cycleStart = a
    while cycleStart < b-1:
        val = array[cycleStart].copy()
        pos = cycleStart
        i = cycleStart+1
        UniV_markArray(0, pos)
        while i < b:
            if array[i] < val:
                pos += 1
                UniV_markArray(0, pos)
            i += 1
        if pos == cycleStart:
            cycleStart += 1
            continue
        while val == array[pos]:
            pos += 1
            UniV_markArray(0, pos)
        tmp = array[pos].copy()
        array[pos].write(val)
        val = tmp.copy()
        while pos != cycleStart:
            pos = cycleStart
            i = cycleStart+1
            UniV_markArray(0, pos)
            while i < b:
                if array[i] < val:
                    pos += 1
                    UniV_markArray(0, pos)
                i += 1
            while val == array[pos]:
                pos += 1
                UniV_markArray(0, pos)
            tmp = array[pos].copy()
            array[pos].write(val)
            val = tmp.copy()
        cycleStart += 1


@Sort("Selection Sorts", "Cycle Sort", "Cycle Sort")
def cycleSortRun(array):
    cycleSort(array, 0, len(array))
