class AeosQuickSort:
    def __init__(this):
        this.indices = None

    def medianOfFewUnique(this, array, start, end):
        i = start
        read = 0
        while read == 0:
            i += 1
            read = compareValues(array[start], array[i])
        if read < 0:
            return i
        else:
            return start

    def rotate(this, array, a, ll, rl):
        j = a+ll
        k = 0
        while k < rl:
            this.aux[k].write(array[j])
            k += 1
            j += 1
        k = a+ll
        while k > a:
            j -= 1
            k -= 1
            array[j].write(array[k])
        j = 0
        while j < rl:
            array[k].write(this.aux[j])
            k += 1
            j += 1

    def partition(this, array, a, b, sqrt, piv):
        smalls = 0
        larges = 0
        smallBlocks = 0
        blockCnt = 0
        i = a
        while i < b:
            if array[i] < piv:
                if larges != 0:
                    array[a+blockCnt*sqrt+smalls].write(array[i])
                smalls += 1
                if smalls == sqrt:
                    smalls = 0
                    this.indices[blockCnt].write(smallBlocks)
                    blockCnt += 1
                    smallBlocks += 1
            else:
                this.aux[larges].write(array[i])
                larges += 1
                if larges == sqrt:
                    j = i
                    k = i-sqrt
                    while k >= a+blockCnt*sqrt:
                        array[j].write(array[k])
                        j -= 1
                        k -= 1
                    k = sqrt-1
                    while k >= 0:
                        array[j].write(this.aux[k])
                        k -= 1
                        j -= 1
                    larges = 0
                    this.indices[blockCnt].write(-1)
                    blockCnt += 1
            i += 1
        j = b-1
        k = larges-1
        while k >= 0:
            array[j].write(this.aux[k])
            k -= 1
            j -= 1
        if smallBlocks == blockCnt:
            return smallBlocks*sqrt+smalls
        if smallBlocks == 0:
            if smalls != 0:
                this.rotate(array, a, blockCnt*sqrt, smalls)
            return smalls
        largeFinalPos = smallBlocks
        for i in range(blockCnt):
            if this.indices[i] == -1:
                this.indices[i].write(largeFinalPos)
                largeFinalPos += 1
        i = 0
        while i < blockCnt and this.indices[i] == i:
            i += 1
        while i < blockCnt:
            j = a+i*sqrt
            k = 0
            while k < sqrt:
                this.aux[k].write(array[j])
                j += 1
                k += 1
            to = this.indices[i].readInt()
            current = i
            next = i
            while this.indices[next] != current:
                next += 1
            while next != to:
                j = a+next*sqrt
                k = a+current*sqrt
                while j < a+(next+1)*sqrt:
                    array[k].write(array[j])
                    j += 1
                    k += 1
                this.indices[current].write(current)
                current = next
                next = i
                while this.indices[next] != current:
                    next += 1
            j = a+next*sqrt
            k = a+current*sqrt
            while j < a+(next+1)*sqrt:
                array[k].write(array[j])
                j += 1
                k += 1
            this.indices[current].write(current)
            j = 0
            k = a+to*sqrt
            while j < sqrt:
                array[k].write(this.aux[j])
                j += 1
                k += 1
            this.indices[to].write(to)
            while True:
                i += 1
                if not (i < blockCnt and this.indices[i] == i):
                    break
        if smalls != 0:
            this.rotate(array, a+smallBlocks*sqrt,
                        (blockCnt-smallBlocks)*sqrt, smalls)
        return smallBlocks*sqrt+smalls

    def sortRec(this, array, a, b, sqrt, badPartition):
        while b-a >= 16:
            pivPos: int
            if badPartition == 0:
                pivPos = medianOf9(array, a, b)
            elif badPartition > 0:
                len = b-a
                if (len & 1) == 0:
                    len -= 1
                pivPos = medianOfMedians(array, a, len)
            else:
                pivPos = this.medianOfFewUnique(array, a, b)
                badPartition = ~badPartition
            piv = array[pivPos].readInt()
            pivPos = this.partition(array, a, b, sqrt, piv)
            newEnd = a+pivPos
            newStart = newEnd
            while newStart < b and array[newStart] == piv:
                newStart += 1
            len1 = newEnd-a
            len2 = b-newStart
            if len1 > len2:
                badPartition += int(len1 > 8*len2)
                this.sortRec(array, newStart, b, sqrt, badPartition)
                b = newEnd
            elif len2 > 8*len1:
                if len1 == 0:
                    badPartition = ~badPartition
                else:
                    badPartition += 1
                    this.sortRec(array, a, newEnd, sqrt, badPartition)
                    a = newStart
            else:
                this.sortRec(array, a, newEnd, sqrt, badPartition)
                a = newStart
        insertionSort(array, a, b)

    def sort(this, array, a, b):
        if b-a < 16:
            insertionSort(array, a, b)
            return
        lgSqrt = 2
        while 1 << (lgSqrt << 1) < b-a:
            lgSqrt += 1
        sqrt = 1 << lgSqrt
        this.aux = sortingVisualizer.createValueArray(sqrt)
        this.indices = sortingVisualizer.createValueArray((b-a)//sqrt)
        sortingVisualizer.setNonOrigAux([this.indices])
        this.sortRec(array, a, b, sqrt, 0)


@Sort("Quick Sorts", "Aeos QuickSort", "Aeos Quick")
def aeosQuicksortRun(array):
    AeosQuickSort().sort(array, 0, len(array))
