class AmericanFlagSort:
    def __init__(this, buckets):
        if buckets is None:
            this.buckets = sortingVisualizer.getUserInput(
                "Insert bucket count: ", "128", parseInt)
        else:
            this.buckets = buckets

    def sorter(this, array, a, b, d):
        count = sortingVisualizer.createValueArray(this.buckets)
        offset = sortingVisualizer.createValueArray(this.buckets)
        sortingVisualizer.setNonOrigAux([count, offset])
        digit: int
        i = a
        while i < b:
            digit = array[i].readDigit(d, this.buckets)
            count[digit] += 1
            i += 1
        offset[0].write(a)
        i = 1
        while i < this.buckets:
            offset[i].write(count[i-1]+offset[i-1])
            i += 1
        for v in range(this.buckets):
            while count[v] > 0:
                origin = offset[v].readInt()
                from_ = origin
                num = array[from_].copy()
                array[from_].write(-1)
                while True:
                    digit = num.readDigit(d, this.buckets)
                    to = offset[digit].readInt()
                    offset[digit] += 1
                    count[digit] -= 1
                    temp = array[to].copy()
                    array[to].write(num)
                    num = temp.copy()
                    from_ = to
                    if not (from_ != origin):
                        break
        UniV_removeAux(count)
        if d > 0:
            for i in range(this.buckets):
                begin = offset[i-1].readInt()if i > 0 else a
                end = offset[i].readInt()
                if end-begin > 1:
                    this.sorter(array, begin, end, d-1)

    def sort(this, array, a, b):
        m = findHighestPower(array, a, b, this.buckets)
        this.sorter(array, a, b, m)


@Sort("Distribution Sorts", "American Flag Sort", "American Flag Sort")
def americanFlagSortRun(array):
    AmericanFlagSort(None).sort(array, 0, len(array))
