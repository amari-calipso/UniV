class ShelfSort:
    SMALL_SORT = 4

    def __init__(this):
        this.scratch = None
        this.indicesA = None
        this.indicesB = None

    def smallSort(this, array, start):
        a = array[start].copy()
        b = array[start+1].copy()
        c = array[start+2].copy()
        d = array[start+3].copy()
        a2: Value
        b2: Value
        c2: Value
        d2: Value
        if b < a:
            a2 = b
            b2 = a
        else:
            a2 = a
            b2 = b
        if d < c:
            c2 = d
            d2 = c
        else:
            c2 = c
            d2 = d
        if b2 <= c2:
            array[start].write(a2)
            array[start+1].write(b2)
            array[start+2].write(c2)
            array[start+3].write(d2)
            return
        b3: Value
        c3: Value
        if a2 <= c2:
            b3 = c2
            array[start].write(a2)
        else:
            b3 = a2
            array[start].write(c2)
        if b2 <= d2:
            c3 = b2
            array[start+3].write(d2)
        else:
            c3 = d2
            array[start+3].write(b2)
        if a2 <= d2:
            array[start+1].write(b3)
            array[start+2].write(c3)
        else:
            array[start+1].write(c3)
            array[start+2].write(b3)

    def mergePair(this, array, start1, start2, output, oStart, n):
        i1 = n
        i2 = n
        i: int
        i = n*2+1
        while i1 >= 0 and i2 >= 0:
            x = array[start1+i1]
            y = array[start2+i2]
            if y < x:
                i1 -= 1
                output[oStart+i].write(x)
            else:
                i2 -= 1
                output[oStart+i].write(y)
            i -= 1
        if i1 >= 0:
            arrayCopy(array, start1, output, oStart, i1+1)
        else:
            arrayCopy(array, start2, output, oStart, i2+1)

    def blockMerge(this, array, start, scratch, iStart, bCount1, bCount2, bSize):
        ii1 = bCount1-1
        ii2 = bCount2-1
        bId1 = this.indicesA[iStart+ii1].readInt()
        bId2 = this.indicesA[iStart+bCount1+ii2].readInt()
        p1 = start+bId1*bSize
        p2 = start+(bCount1+bId2)*bSize
        outBCount = bCount1+bCount2-2
        clearBId = 0
        nextClearBId = 0
        i = bSize*2-1
        i1 = bSize-1
        i2 = i1
        outOffs = 0
        output = scratch
        lastOfFirst = array[p1+(this.indicesA[iStart].readInt()*bSize)+bSize-1]
        firstOfLast = array[start +
                            (bCount1+this.indicesA[iStart+bCount1].readInt())*bSize]
        if lastOfFirst <= firstOfLast:
            for i in range(bCount1):
                this.indicesB[iStart+i].write(this.indicesA[iStart+i])
            i = bCount1
            while i < bCount1+bCount2:
                this.indicesB[iStart+i].write(this.indicesA[iStart+i]+bCount1)
                i += 1
            return
        while True:
            while i1 >= 0 and i2 >= 0 and i >= 0:
                x = array[p1+i1]
                y = array[p2+i2]
                if y < x:
                    output[outOffs+i].write(x)
                    i1 -= 1
                else:
                    output[outOffs+i].write(y)
                    i2 -= 1
                i -= 1
            if i < 0:
                outOffs = start+nextClearBId*bSize
                output = array
                outBCount -= 1
                this.indicesB[iStart+outBCount].write(nextClearBId)
                i = bSize-1
            if i1 < 0:
                nextClearBId = bId1
                ii1 -= 1
                if ii1 < 0:
                    while True:
                        while i2 >= 0 and i >= 0:
                            output[outOffs+i].write(array[p2+i2])
                            i2 -= 1
                            i -= 1
                        if i < 0:
                            clearBId = nextClearBId
                            outOffs = start+nextClearBId*bSize
                            output = array
                            if i2 >= 0:
                                this.indicesB[iStart +
                                              outBCount].write(nextClearBId)
                            outBCount -= 1
                            i = bSize-1
                        if i2 < 0:
                            nextClearBId = bCount1+bId2
                            ii2 -= 1
                            if ii2 < 0:
                                arrayCopy(scratch, 0, output, outOffs, bSize)
                                arrayCopy(scratch, bSize, array,
                                          start+nextClearBId*bSize, bSize)
                                last = bCount1+bCount2-1
                                this.indicesB[iStart+last-1].write(clearBId)
                                this.indicesB[iStart+last].write(nextClearBId)
                                return
                            bId2 = this.indicesA[iStart+bCount1+ii2].readInt()
                            p2 = start+(bCount1+bId2)*bSize
                            i2 = bSize-1
                bId1 = this.indicesA[iStart+ii1].readInt()
                p1 = start+bId1*bSize
                i1 = bSize-1
            if i2 < 0:
                nextClearBId = bCount1+bId2
                ii2 -= 1
                if ii2 < 0:
                    while True:
                        while i1 >= 0 and i >= 0:
                            output[outOffs+i].write(array[p1+i1])
                            i1 -= 1
                            i -= 1
                        if i < 0:
                            clearBId = nextClearBId
                            outOffs = start+nextClearBId*bSize
                            output = array
                            if i1 >= 0:
                                this.indicesB[iStart +
                                              outBCount].write(nextClearBId)
                            outBCount -= 1
                            i = bSize-1
                        if i1 < 0:
                            nextClearBId = bId1
                            ii1 -= 1
                            if ii1 < 0:
                                arrayCopy(scratch, 0, output, outOffs, bSize)
                                arrayCopy(scratch, bSize, array,
                                          start+nextClearBId*bSize, bSize)
                                last = bCount1+bCount2-1
                                this.indicesB[iStart+last-1].write(clearBId)
                                this.indicesB[iStart+last].write(nextClearBId)
                                return
                            bId1 = this.indicesA[iStart+ii1].readInt()
                            p1 = start+bId1*bSize
                            i1 = bSize-1
                bId2 = this.indicesA[iStart+bCount1+ii2].readInt()
                p2 = start+(bCount1+bId2)*bSize
                i2 = bSize-1

    def finalBlockSorting(this, array, start, scratch, blocks, bSize):
        for b in range(blocks):
            ix = this.indicesA[b].readInt()
            if ix != b:
                arrayCopy(array, start+b*bSize, scratch, 0, bSize)
                emptyBlock = b
                while ix != b:
                    arrayCopy(array, start+ix*bSize, array,
                              start+emptyBlock*bSize, bSize)
                    this.indicesA[emptyBlock].write(emptyBlock)
                    emptyBlock = ix
                    ix = this.indicesA[ix].readInt()
                arrayCopy(scratch, 0, array, start+emptyBlock*bSize, bSize)
                this.indicesA[emptyBlock].write(emptyBlock)

    def sort(this, array, start, size):
        logSize = 0
        v = size
        while v > 0:
            logSize += 1
            v //= 2
        scratchSize = 1 << (2+(logSize+1)//2)
        i = 0
        while i < size:
            this.smallSort(array, start+i)
            i += this.SMALL_SORT
        this.scratch = sortingVisualizer.createValueArray(scratchSize)
        this.indicesA = sortingVisualizer.createValueArray(scratchSize)
        this.indicesB = sortingVisualizer.createValueArray(scratchSize)
        sortingVisualizer.setNonOrigAux([this.indicesA, this.indicesB])
        sortedZoneSize = this.SMALL_SORT
        runLen: int
        i: int
        while sortedZoneSize < scratchSize//2:
            runLen = sortedZoneSize
            sortedZoneSize *= 2
            i = 0
            while i < size:
                p1 = start+i
                p2 = start+i+runLen
                p3 = p2+runLen
                p4 = p3+runLen
                less1 = array[p2-1] <= array[p2]
                less2 = array[p4-1] <= array[p4]
                if not less1:
                    this.mergePair(array, p1, p2, this.scratch, 0, runLen-1)
                if not less2:
                    this.mergePair(array, p3, p4, this.scratch,
                                   sortedZoneSize, runLen-1)
                if less1 or less2:
                    if not less1:
                        arrayCopy(array, p3, this.scratch,
                                  sortedZoneSize, sortedZoneSize)
                    elif not less2:
                        arrayCopy(array, p1, this.scratch, 0, sortedZoneSize)
                    elif array[p1+sortedZoneSize-1] <= array[p3]:
                        i += sortedZoneSize*2
                        continue
                    else:
                        arrayCopy(array, p1, this.scratch, 0, sortedZoneSize*2)
                this.mergePair(this.scratch, 0, sortedZoneSize,
                               array, p1, sortedZoneSize-1)
                i += sortedZoneSize*2
            sortedZoneSize *= 2
        bSize = scratchSize//2
        total = size//bSize
        blocksPerRun = sortedZoneSize//bSize
        j: int
        blocks1: int
        blocks2: int
        i = 0
        while i < total:
            for j in range(blocksPerRun):
                this.indicesA[i+j].write(j)
            i += blocksPerRun
        while sortedZoneSize <= (size//2):
            runLen = sortedZoneSize
            blocks1 = sortedZoneSize//bSize
            blocks2 = blocks1
            sortedZoneSize *= 2
            i = 0
            while i < total:
                this.blockMerge(array, start+i*bSize,
                                this.scratch, i, blocks1, blocks2, bSize)
                i += blocks1+blocks2
            for i in range(len(this.indicesA)):
                tmp = this.indicesA[i].copy()
                this.indicesA[i].write(this.indicesB[i])
                this.indicesB[i].write(tmp)
        this.finalBlockSorting(array, start, this.scratch,
                               sortedZoneSize//bSize, bSize)


@Sort("Block Merge Sorts", "Shelf Sort", "Shelf Sort")
def shelfSortRun(array):
    ShelfSort().sort(array, 0, len(array))
