class PatienceSort:
    def __init__(this):
        this.tmp = None
        this.loc = None
        this.pa = None
        this.pb = None
        this.heap = None

    def pileSearch(this, array, b, val) -> int:
        a = 0
        while a < b:
            m = (a+b)//2
            if array[m] <= val:
                b = m
            else:
                a = m+1
        return a

    def sort(this, array, length):
        this.tmp = sortingVisualizer.createValueArray(length)
        this.loc = sortingVisualizer.createValueArray(length)
        sortingVisualizer.setNonOrigAux([this.loc])
        size = 1
        this.tmp[0].write(array[0])
        i = 1
        while i < length:
            l = this.pileSearch(this.tmp, size, array[i])
            this.loc[i].write(l)
            this.tmp[l].write(array[i])
            if l == size:
                size += 1
            i += 1
        if size > 1:
            this.pa = sortingVisualizer.createValueArray(size)
            this.pb = sortingVisualizer.createValueArray(size)
            this.heap = sortingVisualizer.createValueArray(size)
            sortingVisualizer.setNonOrigAux([this.pa, this.pb, this.heap])
            i = 0
            while i < length:
                this.pa[this.loc[i].readInt()] += 1
                i += 1
            i = 1
            while i < size:
                this.pa[i] += this.pa[i-1]
                i += 1
            bidirArrayCopy(this.pa, 0, this.pb, 0, size)
            i = length-1
            while i >= 0:
                l = this.loc[i].readInt()
                this.pa[l] -= 1
                this.tmp[this.pa[l].readInt()].write(array[i])
                i -= 1
            kWayMerge.kWayMerge(this.tmp, array, this.heap,
                                this.pa, this.pb, size)


@Sort("Tree Sorts", "Patience Sort", "Patience Sort")
def patienceSortRun(array):
    PatienceSort().sort(array, len(array))
