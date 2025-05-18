class GrailSort:
    GRAIL_STATIC_EXT_BUF_LEN = 512
    extBuffer = None
    extBufferLen = 0
    Subarray_LEFT = 0
    Subarray_RIGHT = 1

    def compareVal(this, a, b):
        return compareValues(a, b)

    def grailSwap(this, array, a, b):
        array[a].swap(array[b])

    def grailBlockSwap(this, array, a, b, blockLen):
        i = 0
        while i < blockLen:
            this.grailSwap(array, a+i, b+i)
            i += 1

    def grailInsertSort(this, array, start, length):
        item = 1
        while item < length:
            left = start+item-1
            right = start+item
            while left >= start and array[left] > array[right]:
                this.grailSwap(array, left, right)
                left -= 1
                right -= 1
            item += 1

    def grailBinarySearchLeft(this, array, start, length, target):
        left = 0
        right = length
        while left < right:
            middle = left+((right-left)//2)
            if array[start+middle] < target:
                left = middle+1
            else:
                right = middle
        return left

    def grailBinarySearchRight(this, array, start, length, target):
        left = 0
        right = length
        while left < right:
            middle = left+((right-left)//2)
            if array[start+middle] > target:
                right = middle
            else:
                left = middle+1
        return right

    def grailCollectKeys(this, array, start, length, idealKeys):
        rotate = this.grailRotate
        keysFound = 1
        firstKey = 0
        currKey = 1
        while currKey < length and keysFound < idealKeys:
            insertPos: int
            insertPos = this.grailBinarySearchLeft(
                array, start+firstKey, keysFound, array[start+currKey].read())
            if insertPos == keysFound or array[start+currKey] != array[start+firstKey+insertPos]:
                rotate(array, start+firstKey,
                                 keysFound, currKey-(firstKey+keysFound))
                firstKey = currKey-keysFound
                rotate(array, start+firstKey +
                                 insertPos, keysFound-insertPos, 1)
                keysFound += 1
            currKey += 1
        rotate(array, start, firstKey, keysFound)
        return keysFound

    def grailPairwiseSwaps(this, array, start, length):
        index = 1
        while index < length:
            left = start+index-1
            right = start+index
            if array[left] > array[right]:
                this.grailSwap(array, left-2, right)
                this.grailSwap(array, right-2, left)
            else:
                this.grailSwap(array, left-2, left)
                this.grailSwap(array, right-2, right)
            index += 2
        left = start+index-1
        if left < start+length:
            this.grailSwap(array, left-2, left)

    def grailPairwiseWrites(this, array, start, length):
        index = 1
        while index < length:
            left = start+index-1
            right = start+index
            if array[left] > array[right]:
                array[left-2].write(array[right])
                array[right-2].write(array[left])
            else:
                array[left-2].write(array[left])
                array[right-2].write(array[right])
            index += 2
        left = start+index-1
        if left < start+length:
            array[left-2].write(array[left])

    def grailMergeForwards(this, array, start, leftLen, rightLen, bufferOffset):
        left = start
        middle = start+leftLen
        right = middle
        end = middle+rightLen
        buffer = start-bufferOffset
        while right < end:
            if left == middle or array[left] > array[right]:
                this.grailSwap(array, buffer, right)
                right += 1
            else:
                this.grailSwap(array, buffer, left)
                left += 1
            buffer += 1
        if buffer != left:
            this.grailBlockSwap(array, buffer, left, middle-left)

    def grailMergeOutOfPlace(this, array, start, leftLen, rightLen, bufferOffset):
        left = start
        middle = start+leftLen
        right = middle
        end = middle+rightLen
        buffer = start-bufferOffset
        while right < end:
            if left == middle or array[left] > array[right]:
                array[buffer].write(array[right])
                right += 1
            else:
                array[buffer].write(array[left])
                left += 1
            buffer += 1
        if buffer != left:
            while left < middle:
                array[buffer].write(array[left])
                buffer += 1
                left += 1

    def grailMergeBackwards(this, array, start, leftLen, rightLen, bufferOffset):
        end = start-1
        left = end+leftLen
        middle = left
        right = middle+rightLen
        buffer = right+bufferOffset
        while left > end:
            if right == middle or array[left] > array[right]:
                this.grailSwap(array, buffer, left)
                left -= 1
            else:
                this.grailSwap(array, buffer, right)
                right -= 1
            buffer -= 1
        if right != buffer:
            while right > middle:
                this.grailSwap(array, buffer, right)
                buffer -= 1
                right -= 1

    def grailBuildInPlace(this, array, start, length, currentLen, bufferLen):
        rotate = this.grailRotate
        mergeLen = currentLen
        while mergeLen < bufferLen:
            fullMerge = 2*mergeLen
            mergeEnd = start+length-fullMerge
            bufferOffset = mergeLen
            mergeIndex = start
            while mergeIndex <= mergeEnd:
                this.grailMergeForwards(
                    array, mergeIndex, mergeLen, mergeLen, bufferOffset)
                mergeIndex += fullMerge
            leftOver = length-(mergeIndex-start)
            if leftOver > mergeLen:
                this.grailMergeForwards(
                    array, mergeIndex, mergeLen, leftOver-mergeLen, bufferOffset)
            else:
                rotate(array, mergeIndex-mergeLen,
                                 mergeLen, leftOver)
            start -= mergeLen
            mergeLen *= 2
        fullMerge = 2*bufferLen
        lastBlock = length % fullMerge
        lastOffset = start+length-lastBlock
        if lastBlock <= bufferLen:
            rotate(array, lastOffset, lastBlock, bufferLen)
        else:
            this.grailMergeBackwards(
                array, lastOffset, bufferLen, lastBlock-bufferLen, bufferLen)
        mergeIndex = lastOffset-fullMerge
        while mergeIndex >= start:
            this.grailMergeBackwards(
                array, mergeIndex, bufferLen, bufferLen, bufferLen)
            mergeIndex -= fullMerge

    def grailBuildOutOfPlace(this, array, start, length, bufferLen, extLen):
        arrayCopy(array, start-extLen, this.extBuffer, 0, extLen)
        this.grailPairwiseWrites(array, start, length)
        start -= 2
        mergeLen = 2
        while mergeLen < extLen:
            fullMerge = 2*mergeLen
            mergeEnd = start+length-fullMerge
            bufferOffset = mergeLen
            mergeIndex = start
            while mergeIndex <= mergeEnd:
                this.grailMergeOutOfPlace(
                    array, mergeIndex, mergeLen, mergeLen, bufferOffset)
                mergeIndex += fullMerge
            leftOver = length-(mergeIndex-start)
            if leftOver > mergeLen:
                this.grailMergeOutOfPlace(
                    array, mergeIndex, mergeLen, leftOver-mergeLen, bufferOffset)
            else:
                arrayCopy(array, mergeIndex, array,
                          mergeIndex-mergeLen, leftOver)
            start -= mergeLen
            mergeLen *= 2
        arrayCopy(this.extBuffer, 0, array, start+length, extLen)
        this.grailBuildInPlace(array, start, length, mergeLen, bufferLen)

    def grailBuildBlocks(this, array, start, length, bufferLen):
        if this.extBuffer is not None:
            extLen: int
            if bufferLen < this.extBufferLen:
                extLen = bufferLen
            else:
                extLen = 1
                while extLen*2 <= this.extBufferLen:
                    extLen *= 2
            this.grailBuildOutOfPlace(array, start, length, bufferLen, extLen)
        else:
            this.grailPairwiseSwaps(array, start, length)
            this.grailBuildInPlace(array, start-2, length, 2, bufferLen)

    def grailBlockSelectSort(this, array, firstKey, start, medianKey, blockCount, blockLen):
        for firstBlock in range(blockCount):
            selectBlock = firstBlock
            currBlock = firstBlock+1
            while currBlock < blockCount:
                compare: int
                compare = this.compareVal(
                    array[start+(currBlock*blockLen)], array[start+(selectBlock*blockLen)])
                if compare < 0 or (compare == 0 and array[firstKey+currBlock] < array[firstKey+selectBlock]):
                    selectBlock = currBlock
                currBlock += 1
            if selectBlock != firstBlock:
                this.grailBlockSwap(
                    array, start+(firstBlock*blockLen), start+(selectBlock*blockLen), blockLen)
                this.grailSwap(array, firstKey+firstBlock,
                               firstKey+selectBlock)
                if medianKey == firstBlock:
                    medianKey = selectBlock
                else:
                    if medianKey == selectBlock:
                        medianKey = firstBlock
        return medianKey

    def grailInPlaceBufferReset(this, array, start, length, bufferOffset):
        buffer = start+length-1
        index = buffer-bufferOffset
        while buffer >= start:
            this.grailSwap(array, index, buffer)
            buffer -= 1
            index -= 1

    def grailOutOfPlaceBufferReset(this, array, start, length, bufferOffset):
        buffer = start+length-1
        index = buffer-bufferOffset
        while buffer >= start:
            array[buffer].write(array[index])
            buffer -= 1
            index -= 1

    def grailInPlaceBufferRewind(this, array, start, leftBlock, buffer):
        while leftBlock >= start:
            this.grailSwap(array, buffer, leftBlock)
            buffer -= 1
            leftBlock -= 1

    def grailOutOfPlaceBufferRewind(this, array, start, leftBlock, buffer):
        while leftBlock >= start:
            array[buffer].write(array[leftBlock])
            buffer -= 1
            leftBlock -= 1

    def grailGetSubarray(this, array, currentKey, medianKey):
        if array[currentKey] < array[medianKey]:
            return this.Subarray_LEFT
        else:
            return this.Subarray_RIGHT

    def grailCountLastMergeBlocks(this, array, offset, blockCount, blockLen):
        blocksToMerge = 0
        lastRightFrag = offset+(blockCount*blockLen)
        prevLeftBlock = lastRightFrag-blockLen
        while (blocksToMerge < blockCount) and (array[lastRightFrag] < array[prevLeftBlock]):
            blocksToMerge += 1
            prevLeftBlock -= blockLen
        return blocksToMerge

    def grailSmartMerge(this, array, start, leftLen, leftOrigin, rightLen, bufferOffset):
        left = start
        middle = start+leftLen
        right = middle
        end = middle+rightLen
        buffer = start-bufferOffset
        if leftOrigin == this.Subarray_LEFT:
            while left < middle and right < end:
                if array[left] <= array[right]:
                    this.grailSwap(array, buffer, left)
                    left += 1
                else:
                    this.grailSwap(array, buffer, right)
                    right += 1
                buffer += 1
        else:
            while left < middle and right < end:
                if array[left] < array[right]:
                    this.grailSwap(array, buffer, left)
                    left += 1
                else:
                    this.grailSwap(array, buffer, right)
                    right += 1
                buffer += 1
        if left < middle:
            this.currBlockLen = middle-left
            this.grailInPlaceBufferRewind(array, left, middle-1, end-1)
        else:
            this.currBlockLen = end-right
            if leftOrigin == this.Subarray_LEFT:
                this.currBlockOrigin = this.Subarray_RIGHT
            else:
                this.currBlockOrigin = this.Subarray_LEFT

    def grailSmartLazyMerge(this, array, start, leftLen, leftOrigin, rightLen):
        rotate = this.grailRotate
        middle = start+leftLen
        mergeLen: int
        if leftOrigin == this.Subarray_LEFT:
            if array[middle-1] > array[middle]:
                while leftLen != 0:
                    mergeLen = this.grailBinarySearchLeft(
                        array, middle, rightLen, array[start])
                    if mergeLen != 0:
                        rotate(array, start, leftLen, mergeLen)
                        start += mergeLen
                        rightLen -= mergeLen
                        middle += mergeLen
                    if rightLen == 0:
                        this.currBlockLen = leftLen
                        return
                    else:
                        while True:
                            start += 1
                            leftLen -= 1
                            if not (leftLen != 0 and array[start] <= array[middle]):
                                break
        else:
            if array[middle-1] >= array[middle]:
                while leftLen != 0:
                    mergeLen = this.grailBinarySearchRight(
                        array, middle, rightLen, array[start])
                    if mergeLen != 0:
                        rotate(array, start, leftLen, mergeLen)
                        start += mergeLen
                        rightLen -= mergeLen
                        middle += mergeLen
                    if rightLen == 0:
                        this.currBlockLen = leftLen
                        return
                    else:
                        while True:
                            start += 1
                            leftLen -= 1
                            if not (leftLen != 0 and array[start] < array[middle]):
                                break
        this.currBlockLen = rightLen
        if leftOrigin == this.Subarray_LEFT:
            this.currBlockOrigin = this.Subarray_RIGHT
        else:
            this.currBlockOrigin = this.Subarray_LEFT

    def grailSmartMergeOutOfPlace(this, array, start, leftLen, leftOrigin, rightLen, bufferOffset):
        left = start
        middle = start+leftLen
        right = middle
        end = middle+rightLen
        buffer = start-bufferOffset
        if leftOrigin == this.Subarray_LEFT:
            while left < middle and right < end:
                if array[left] <= array[right]:
                    array[buffer].write(array[left])
                    left += 1
                else:
                    array[buffer].write(array[right])
                    right += 1
                buffer += 1
        else:
            while left < middle and right < end:
                if array[left] < array[right]:
                    array[buffer].write(array[left])
                    left += 1
                else:
                    array[buffer].write(array[right])
                    right += 1
                buffer += 1
        if left < middle:
            this.currBlockLen = middle-left
            this.grailOutOfPlaceBufferRewind(array, left, middle-1, end-1)
        else:
            this.currBlockLen = end-right
            if leftOrigin == this.Subarray_LEFT:
                this.currBlockOrigin = this.Subarray_RIGHT
            else:
                this.currBlockOrigin = this.Subarray_LEFT

    def grailMergeBlocks(this, array, firstKey, medianKey, start, blockCount, blockLen, lastMergeBlocks, lastLen):
        nextBlock = start+blockLen
        this.currBlockLen = blockLen
        this.currBlockOrigin = this.grailGetSubarray(
            array, firstKey, medianKey)
        keyIndex = 1
        while keyIndex < blockCount:
            currBlock = nextBlock-this.currBlockLen
            nextBlockOrigin = this.grailGetSubarray(
                array, firstKey+keyIndex, medianKey)
            if nextBlockOrigin == this.currBlockOrigin:
                buffer = currBlock-blockLen
                this.grailBlockSwap(
                    array, buffer, currBlock, this.currBlockLen)
                this.currBlockLen = blockLen
            else:
                this.grailSmartMerge(
                    array, currBlock, this.currBlockLen, this.currBlockOrigin, blockLen, blockLen)
            keyIndex += 1
            nextBlock += blockLen
        currBlock = nextBlock-this.currBlockLen
        buffer = currBlock-blockLen
        if lastLen != 0:
            if this.currBlockOrigin == this.Subarray_RIGHT:
                this.grailBlockSwap(
                    array, buffer, currBlock, this.currBlockLen)
                currBlock = nextBlock
                this.currBlockLen = blockLen*lastMergeBlocks
                this.currBlockOrigin = this.Subarray_LEFT
            else:
                this.currBlockLen += blockLen*lastMergeBlocks
            this.grailMergeForwards(
                array, currBlock, this.currBlockLen, lastLen, blockLen)
        else:
            this.grailBlockSwap(array, buffer, currBlock, this.currBlockLen)

    def grailLazyMergeBlocks(this, array, firstKey, medianKey, start, blockCount, blockLen, lastMergeBlocks, lastLen):
        nextBlock = start+blockLen
        this.currBlockLen = blockLen
        this.currBlockOrigin = this.grailGetSubarray(
            array, firstKey, medianKey)
        keyIndex = 1
        while keyIndex < blockCount:
            currBlock = nextBlock-this.currBlockLen
            nextBlockOrigin: int
            nextBlockOrigin = this.grailGetSubarray(
                array, firstKey+keyIndex, medianKey)
            if nextBlockOrigin == this.currBlockOrigin:
                this.currBlockLen = blockLen
            else:
                this.grailSmartLazyMerge(
                    array, currBlock, this.currBlockLen, this.currBlockOrigin, blockLen)
            keyIndex += 1
            nextBlock += blockLen
        currBlock = nextBlock-this.currBlockLen
        if lastLen != 0:
            if this.currBlockOrigin == this.Subarray_RIGHT:
                currBlock = nextBlock
                this.currBlockLen = blockLen*lastMergeBlocks
                this.currBlockOrigin = this.Subarray_LEFT
            else:
                this.currBlockLen += blockLen*lastMergeBlocks
            this.grailLazyMerge(array, currBlock, this.currBlockLen, lastLen)

    def grailMergeBlocksOutOfPlace(this, array, firstKey, medianKey, start, blockCount, blockLen, lastMergeBlocks, lastLen):
        nextBlock = start+blockLen
        this.currBlockLen = blockLen
        this.currBlockOrigin = this.grailGetSubarray(
            array, firstKey, medianKey)
        buffer: int
        keyIndex = 1
        while keyIndex < blockCount:
            currBlock = nextBlock-this.currBlockLen
            nextBlockOrigin: int
            nextBlockOrigin = this.grailGetSubarray(
                array, firstKey+keyIndex, medianKey)
            if nextBlockOrigin == this.currBlockOrigin:
                buffer = currBlock-blockLen
                arrayCopy(array, currBlock, array, buffer, this.currBlockLen)
                this.currBlockLen = blockLen
            else:
                this.grailSmartMergeOutOfPlace(
                    array, currBlock, this.currBlockLen, this.currBlockOrigin, blockLen, blockLen)
            keyIndex += 1
            nextBlock += blockLen
        currBlock = nextBlock-this.currBlockLen
        buffer = currBlock-blockLen
        if lastLen != 0:
            if this.currBlockOrigin == this.Subarray_RIGHT:
                arrayCopy(array, currBlock, array, buffer, this.currBlockLen)
                currBlock = nextBlock
                this.currBlockLen = blockLen*lastMergeBlocks
                this.currBlockOrigin = this.Subarray_LEFT
            else:
                this.currBlockLen += blockLen*lastMergeBlocks
            this.grailMergeOutOfPlace(
                array, currBlock, this.currBlockLen, lastLen, blockLen)
        else:
            arrayCopy(array, currBlock, array, buffer, this.currBlockLen)

    def grailCombineInPlace(this, array, firstKey, start, length, subarrayLen, blockLen, mergeCount, lastSubarrays, buffer):
        fullMerge = 2*subarrayLen
        blockCount = fullMerge//blockLen
        offset: int
        for mergeIndex in range(mergeCount):
            offset = start+(mergeIndex*fullMerge)
            this.grailInsertSort(array, firstKey, blockCount)
            medianKey = subarrayLen//blockLen
            medianKey = this.grailBlockSelectSort(
                array, firstKey, offset, medianKey, blockCount, blockLen)
            if buffer:
                this.grailMergeBlocks(
                    array, firstKey, firstKey+medianKey, offset, blockCount, blockLen, 0, 0)
            else:
                this.grailLazyMergeBlocks(
                    array, firstKey, firstKey+medianKey, offset, blockCount, blockLen, 0, 0)
        if lastSubarrays != 0:
            offset = start+(mergeCount*fullMerge)
            blockCount = lastSubarrays//blockLen
            this.grailInsertSort(array, firstKey, blockCount+1)
            medianKey = subarrayLen//blockLen
            medianKey = this.grailBlockSelectSort(
                array, firstKey, offset, medianKey, blockCount, blockLen)
            lastFragment = lastSubarrays-(blockCount*blockLen)
            lastMergeBlocks: int
            if lastFragment != 0:
                lastMergeBlocks = this.grailCountLastMergeBlocks(
                    array, offset, blockCount, blockLen)
            else:
                lastMergeBlocks = 0
            smartMerges = blockCount-lastMergeBlocks
            if smartMerges == 0:
                leftLen = lastMergeBlocks*blockLen
                if buffer:
                    this.grailMergeForwards(
                        array, offset, leftLen, lastFragment, blockLen)
                else:
                    this.grailLazyMerge(array, offset, leftLen, lastFragment)
            else:
                if buffer:
                    this.grailMergeBlocks(array, firstKey, firstKey+medianKey,
                                          offset, smartMerges, blockLen, lastMergeBlocks, lastFragment)
                else:
                    this.grailLazyMergeBlocks(
                        array, firstKey, firstKey+medianKey, offset, smartMerges, blockLen, lastMergeBlocks, lastFragment)
        if buffer:
            this.grailInPlaceBufferReset(array, start, length, blockLen)

    def grailCombineOutOfPlace(this, array, firstKey, start, length, subarrayLen, blockLen, mergeCount, lastSubarrays):
        arrayCopy(array, start-blockLen, this.extBuffer, 0, blockLen)
        fullMerge = 2*subarrayLen
        blockCount = fullMerge//blockLen
        offset: int
        for mergeIndex in range(mergeCount):
            offset = start+(mergeIndex*fullMerge)
            this.grailInsertSort(array, firstKey, blockCount)
            medianKey = subarrayLen//blockLen
            medianKey = this.grailBlockSelectSort(
                array, firstKey, offset, medianKey, blockCount, blockLen)
            this.grailMergeBlocksOutOfPlace(
                array, firstKey, firstKey+medianKey, offset, blockCount, blockLen, 0, 0)
        if lastSubarrays != 0:
            offset = start+(mergeCount*fullMerge)
            blockCount = lastSubarrays//blockLen
            this.grailInsertSort(array, firstKey, blockCount+1)
            medianKey = subarrayLen//blockLen
            medianKey = this.grailBlockSelectSort(
                array, firstKey, offset, medianKey, blockCount, blockLen)
            lastFragment = lastSubarrays-(blockCount*blockLen)
            lastMergeBlocks: int
            if lastFragment != 0:
                lastMergeBlocks = this.grailCountLastMergeBlocks(
                    array, offset, blockCount, blockLen)
            else:
                lastMergeBlocks = 0
            smartMerges = blockCount-lastMergeBlocks
            if smartMerges == 0:
                leftLen = lastMergeBlocks*blockLen
                this.grailMergeOutOfPlace(
                    array, offset, leftLen, lastFragment, blockLen)
            else:
                this.grailMergeBlocksOutOfPlace(
                    array, firstKey, firstKey+medianKey, offset, smartMerges, blockLen, lastMergeBlocks, lastFragment)
        this.grailOutOfPlaceBufferReset(array, start, length, blockLen)
        arrayCopy(this.extBuffer, 0, array, start-blockLen, blockLen)

    def grailCombineBlocks(this, array, firstKey, start, length, subarrayLen, blockLen, buffer):
        fullMerge = 2*subarrayLen
        mergeCount = length//fullMerge
        lastSubarrays = length-(fullMerge*mergeCount)
        if lastSubarrays <= subarrayLen:
            length -= lastSubarrays
            lastSubarrays = 0
        if buffer and blockLen <= this.extBufferLen:
            this.grailCombineOutOfPlace(
                array, firstKey, start, length, subarrayLen, blockLen, mergeCount, lastSubarrays)
        else:
            this.grailCombineInPlace(array, firstKey, start, length,
                                     subarrayLen, blockLen, mergeCount, lastSubarrays, buffer)

    def grailLazyMerge(this, array, start, leftLen, rightLen):
        rotate = this.grailRotate
        mergeLen: int
        middle: int
        if leftLen < rightLen:
            middle = start+leftLen
            while leftLen != 0:
                mergeLen = this.grailBinarySearchLeft(
                    array, middle, rightLen, array[start].read())
                if mergeLen != 0:
                    rotate(array, start, leftLen, mergeLen)
                    start += mergeLen
                    rightLen -= mergeLen
                    middle += mergeLen
                if rightLen == 0:
                    break
                else:
                    while True:
                        start += 1
                        leftLen -= 1
                        if not (leftLen != 0 and array[start] <= array[middle]):
                            break
        else:
            end = start+leftLen+rightLen-1
            while rightLen != 0:
                mergeLen = this.grailBinarySearchRight(
                    array, start, leftLen, array[end].read())
                if mergeLen != leftLen:
                    rotate(array, start+mergeLen,
                                     leftLen-mergeLen, rightLen)
                    end -= leftLen-mergeLen
                    leftLen = mergeLen
                if leftLen == 0:
                    break
                else:
                    middle = start+leftLen
                    while True:
                        rightLen -= 1
                        end -= 1
                        if not (rightLen != 0 and array[middle-1] <= array[end]):
                            break

    def grailLazyStableSort(this, array, start, length):
        index = 1
        while index < length:
            left = start+index-1
            right = start+index
            if array[left] > array[right]:
                this.grailSwap(array, left, right)
            index += 2
        mergeLen = 2
        while mergeLen < length:
            fullMerge = 2*mergeLen
            mergeEnd = length-fullMerge
            mergeIndex = 0
            while mergeIndex <= mergeEnd:
                this.grailLazyMerge(
                    array, start+mergeIndex, mergeLen, mergeLen)
                mergeIndex += fullMerge
            leftOver = length-mergeIndex
            if leftOver > mergeLen:
                this.grailLazyMerge(array, start+mergeIndex,
                                    mergeLen, leftOver-mergeLen)
            mergeLen *= 2

    def grailCommonSort(this, array, start, length, extBuffer, extBufferLen):
        if length < 16:
            this.grailInsertSort(array, start, length)
            return
        else:
            blockLen = 1
            while blockLen**2 < length:
                blockLen *= 2
            keyLen = ((length-1)//blockLen)+1
            idealKeys = keyLen+blockLen
            keysFound: int
            keysFound = this.grailCollectKeys(array, start, length, idealKeys)
            idealBuffer: bool
            if keysFound < idealKeys:
                if keysFound < 4:
                    this.grailLazyStableSort(array, start, length)
                    return
                else:
                    keyLen = blockLen
                    blockLen = 0
                    idealBuffer = False
                    while keyLen > keysFound:
                        keyLen //= 2
            else:
                idealBuffer = True
            bufferEnd = blockLen+keyLen
            subarrayLen: int
            if idealBuffer:
                subarrayLen = blockLen
            else:
                subarrayLen = keyLen
            if idealBuffer and extBuffer != None:
                this.extBuffer = extBuffer
                this.extBufferLen = extBufferLen
            this.grailBuildBlocks(array, start+bufferEnd,
                                  length-bufferEnd, subarrayLen)
            while length-bufferEnd > 2*subarrayLen:
                subarrayLen *= 2
                currentBlockLen = blockLen
                scrollingBuffer = idealBuffer
                if not idealBuffer:
                    keyBuffer = keyLen//2
                    if keyBuffer >= (2*subarrayLen)//keyBuffer:
                        currentBlockLen = keyBuffer
                        scrollingBuffer = True
                    else:
                        currentBlockLen = (2*subarrayLen)//keyLen
                this.grailCombineBlocks(array, start, start+bufferEnd, length -
                                        bufferEnd, subarrayLen, currentBlockLen, scrollingBuffer)
            this.grailInsertSort(array, start, bufferEnd)
            this.grailLazyMerge(array, start, bufferEnd, length-bufferEnd)


if __name__ == '__main__':
    grailSort = GrailSort()
    grailSort.grailRotate = sortingVisualizer.getRotationByName("Gries-Mills").lengths


def grailSortInPlace(array, start, length):
    grailSort.extBuffer = None
    grailSort.extBufferLen = 0
    grailSort.grailCommonSort(array, start, length, None, 0)


def grailSortStaticOOP(array, start, length):
    grailSort.extBuffer = sortingVisualizer.createValueArray(
        grailSort.GRAIL_STATIC_EXT_BUF_LEN)
    grailSort.extBufferLen = grailSort.GRAIL_STATIC_EXT_BUF_LEN
    grailSort.grailCommonSort(
        array, start, length, grailSort.extBuffer, grailSort.GRAIL_STATIC_EXT_BUF_LEN)


def grailSortDynamicOOP(array, start, length):
    grailSort.extBufferLen = 1
    while grailSort.extBufferLen**2 < length:
        grailSort.extBufferLen *= 2
    grailSort.extBuffer = sortingVisualizer.createValueArray(
        grailSort.extBufferLen)
    grailSort.grailCommonSort(array, start, length,
                              grailSort.extBuffer, grailSort.extBufferLen)


def grailSortGivenAux(array, start, length, aux):
    grailSort.extBuffer = aux
    grailSort.extBufferLen = len(aux)
    grailSort.grailCommonSort(array, start, length,
                              grailSort.extBuffer, grailSort.extBufferLen)


@Sort("Block Merge Sorts", "Grail Sort", "Grail Sort")
def grailSortRun(array):
    mode: int
    mode = sortingVisualizer.getUserInput("Insert buffer size (0 for in-place, -1 for dynamic)", "0", parseInt)
    rotate = UniV_getUserRotation("Select rotation algorithm (default: Gries-Mills)").lengths
    oldRotate = grailSort.grailRotate
    grailSort.grailRotate = rotate
    match mode:
        case 0:
            grailSortInPlace(array, 0, len(array))
        case-1:
            grailSortDynamicOOP(array, 0, len(array))
        case _:
            grailSort.GRAIL_STATIC_EXT_BUF_LEN = mode
            grailSortStaticOOP(array, 0, len(array))
    grailSort.grailRotate = oldRotate


@Sort("Merge Sorts", "Lazy Stable Sort", "Lazy Stable")
def lazyStableSortRun(array):
    rotate = UniV_getUserRotation("Select rotation algorithm (default: Gries-Mills)").lengths
    oldRotate = grailSort.grailRotate
    grailSort.grailRotate = rotate
    grailSort.grailLazyStableSort(array, 0, len(array))
    grailSort.grailRotate = oldRotate
