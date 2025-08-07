class WikiRange:
    def __init__(this, start, end):
        this.start = start
        this.end = end

    def set(this, start, end):
        this.start = start
        this.end = end

    def length(this):
        return this.end-this.start


class WikiPull:
    def __init__(this):
        this.range = WikiRange(0, 0)
        this.from_ = 0
        this.to = 0
        this.count = 0

    def reset(this):
        this.range.set(0, 0)
        this.from_ = 0
        this.to = 0
        this.count = 0


class WikiIterator:
    def floorPowerOfTwo(this, value):
        x = value
        x = x | (x >> 1)
        x = x | (x >> 2)
        x = x | (x >> 4)
        x = x | (x >> 8)
        x = x | (x >> 16)
        return x-(x >> 1)

    def __init__(this, size, min_level):
        this.size = size
        this.power_of_two = this.floorPowerOfTwo(this.size)
        this.denominator = this.power_of_two//min_level
        this.numerator_step = this.size % this.denominator
        this.decimal_step = this.size//this.denominator
        this.begin()

    def begin(this):
        this.numerator = 0
        this.decimal = 0

    def nextRange(this):
        start = this.decimal
        this.decimal += this.decimal_step
        this.numerator += this.numerator_step
        if this.numerator >= this.denominator:
            this.numerator -= this.denominator
            this.decimal += 1
        return WikiRange(start, this.decimal)

    def finished(this):
        return this.decimal >= this.size

    def nextLevel(this):
        this.decimal_step += this.decimal_step
        this.numerator_step += this.numerator_step
        if this.numerator_step >= this.denominator:
            this.numerator_step -= this.denominator
            this.decimal_step += 1
        return this.decimal_step < this.size

    def length(this):
        return this.decimal_step


class WikiSort:
    def __init__(this, cacheSize, cache, rot):
        this.cache_size = cacheSize
        if this.cache_size != 0:
            if cache is None:
                this.cache = sortingVisualizer.createValueArray(
                    this.cache_size)
            else:
                this.cache = cache
        else:
            this.cache = None
        if rot is None:
            this.internalRotate = UniV_getUserRotation("Select rotation algorithm", "Triple Reversal").indexed
        else:
            this.internalRotate = sortingVisualizer.getRotationByName(rot).indexed

    def binaryFirst(this, array, value, range):
        start = range.start
        end = range.end-1
        while start < end:
            mid = start+((end-start)//2)
            if array[mid] < value:
                start = mid+1
            else:
                end = mid
        if start == range.end-1 and array[start] < value:
            start += 1
        return start

    def binaryLast(this, array, value, range):
        start = range.start
        end = range.end-1
        while start < end:
            mid = start+((end-start)//2)
            if compareIntToValue(value, array[mid]) >= 0:
                start = mid+1
            else:
                end = mid
        if start == range.end-1 and array[start] <= value:
            start += 1
        return start

    def findFirstForward(this, array, value, range, unique):
        if range.length() == 0:
            return range.start
        index: int
        skip: int
        skip = max(range.length()//unique, 1)
        index = range.start+skip
        while array[index-1] < value:
            if index >= range.end-skip:
                return this.binaryFirst(array, value, WikiRange(index, range.end))
            index += skip
        return this.binaryFirst(array, value, WikiRange(index-skip, index))

    def findLastForward(this, array, value, range, unique):
        if range.length() == 0:
            return range.start
        index: int
        skip: int
        skip = max(range.length()//unique, 1)
        index = range.start+skip
        while compareIntToValue(value, array[index-1]) >= 0:
            if index >= range.end-skip:
                return this.binaryLast(array, value, WikiRange(index, range.end))
            index += skip
        return this.binaryLast(array, value, WikiRange(index-skip, index))

    def findFirstBackward(this, array, value, range, unique):
        if range.length() == 0:
            return range.start
        index: int
        skip: int
        skip = max(range.length()//unique, 1)
        index = range.end-skip
        while index > range.start and array[index-1] >= value:
            if index < range.start+skip:
                return this.binaryFirst(array, value, WikiRange(range.start, index))
            index -= skip
        return this.binaryFirst(array, value, WikiRange(index, index+skip))

    def findLastBackward(this, array, value, range, unique):
        if range.length() == 0:
            return range.start
        index: int
        skip: int
        skip = max(range.length()//unique, 1)
        index = range.end-skip
        while index > range.start and compareIntToValue(value, array[index-1]) < 0:
            if index < range.start+skip:
                return this.binaryLast(array, value, WikiRange(range.start, index))
            index -= skip
        return this.binaryLast(array, value, WikiRange(index, index+skip))

    def insertionSort(this, array, range):
        insertionSort(array, range.start, range.end-1)

    def reverse(this, array, range):
        reverse(array, range.start, range.end)

    def rotate(this, array, amount, range, use_cache):
        if range.length() == 0:
            return
        split: int
        if amount >= 0:
            split = range.start+amount
        else:
            split = range.end+amount
        range1: WikiRange
        range2: WikiRange
        range1 = WikiRange(range.start, split)
        range2 = WikiRange(split, range.end)
        if use_cache:
            if range1.length() <= range2.length():
                if range1.length() <= this.cache_size:
                    if this.cache is not None:
                        arrayCopy(array, range1.start,
                                  this.cache, 0, range1.length())
                        arrayCopy(array, range2.start, array,
                                  range1.start, range2.length())
                        arrayCopy(this.cache, 0, array, range1.start +
                                  range2.length(), range1.length())
                    return
            elif range2.length() <= this.cache_size:
                if this.cache is not None:
                    reverseArrayCopy(array, range2.start,
                                     this.cache, 0, range2.length())
                    reverseArrayCopy(array, range1.start, array,
                                     range2.end-range1.length(), range1.length())
                    reverseArrayCopy(this.cache, 0, array,
                                     range1.start, range2.length())
                return
        rotate = this.internalRotate
        rotate(array, range.start, split, range.end)

    def mergeInto(this, from_, A, B, into, at_index):
        A_index: int
        B_index: int
        insert_index: int
        A_last: int
        B_last: int
        A_index = A.start
        B_index = B.start
        insert_index = at_index
        A_last = A.end
        B_last = B.end
        while True:
            if from_[B_index] >= from_[A_index]:
                into[insert_index].write(from_[A_index])
                A_index += 1
                insert_index += 1
                if A_index == A_last:
                    arrayCopy(from_, B_index, into,
                              insert_index, B_last-B_index)
                    break
            else:
                into[insert_index].write(from_[B_index])
                B_index += 1
                insert_index += 1
                if B_index == B_last:
                    arrayCopy(from_, A_index, into,
                              insert_index, A_last-A_index)
                    break

    def mergeExternal(this, array, A, B):
        A_index: int
        B_index: int
        insert_index: int
        A_last: int
        B_last: int
        A_index = 0
        B_index = B.start
        insert_index = A.start
        A_last = A.length()
        B_last = B.end
        if B.length() > 0 and A.length() > 0:
            while True:
                if array[B_index] >= this.cache[A_index]:
                    array[insert_index].write(this.cache[A_index])
                    A_index += 1
                    insert_index += 1
                    if A_index == A_last:
                        break
                else:
                    array[insert_index].write(array[B_index])
                    B_index += 1
                    insert_index += 1
                    if B_index == B_last:
                        break
        if this.cache is not None:
            arrayCopy(this.cache, A_index, array, insert_index, A_last-A_index)

    def mergeInternal(this, array, A, B, buffer):
        A_count = 0
        B_count = 0
        insert = 0
        if B.length() > 0 and A.length() > 0:
            while True:
                if array[B.start+B_count] >= array[buffer.start+A_count]:
                    array[A.start+insert].swap(array[buffer.start+A_count])
                    A_count += 1
                    insert += 1
                    if A_count >= A.length():
                        break
                else:
                    array[A.start+insert].swap(array[B.start+B_count])
                    B_count += 1
                    insert += 1
                    if B_count >= B.length():
                        break
        blockSwap(array, buffer.start+A_count,
                  A.start+insert, A.length()-A_count)

    def mergeInPlace(this, array, A, B):
        A = WikiRange(A.start, A.end)
        B = WikiRange(B.start, B.end)
        mid: int
        amount: int
        while True:
            mid = this.binaryFirst(array, array[A.start].readInt(), B)
            amount = mid-A.end
            this.rotate(array, -amount, WikiRange(A.start, mid), True)
            if B.end == mid:
                break
            B.start = mid
            A.set(A.start+amount, B.start)
            A.start = this.binaryLast(array, array[A.start].readInt(), A)
            if A.length() == 0:
                break

    def netSwap(this, array, order, range, x, y):
        compare: int
        compare = compareValues(
            array[range.start+x].readInt(), array[range.start+y].readInt())
        if compare > 0 or (order[x] > order[y] and compare == 0):
            array[range.start+x].swap(array[range.start+y])
            sortingVisualizer.swap(order, x, y)

    def sort(this, array, len):
        size = len
        if size < 4:
            match size:
                case 3:
                    if array[1] < array[0]:
                        array[0].swap(array[1])
                    if array[2] < array[1]:
                        array[1].swap(array[2])
                        if array[1] < array[0]:
                            array[0].swap(array[1])
                case 2:
                    if array[1] < array[0]:
                        array[0].swap(array[1])
            return
        iterator = WikiIterator(size, 4)
        while not iterator.finished():
            order: list
            order = [0, 1, 2, 3, 4, 5, 6, 7]
            range = iterator.nextRange()
            match range.length():
                case 8:
                    this.netSwap(array, order, range, 0, 1)
                    this.netSwap(array, order, range, 2, 3)
                    this.netSwap(array, order, range, 4, 5)
                    this.netSwap(array, order, range, 6, 7)
                    this.netSwap(array, order, range, 0, 2)
                    this.netSwap(array, order, range, 1, 3)
                    this.netSwap(array, order, range, 4, 6)
                    this.netSwap(array, order, range, 5, 7)
                    this.netSwap(array, order, range, 1, 2)
                    this.netSwap(array, order, range, 5, 6)
                    this.netSwap(array, order, range, 0, 4)
                    this.netSwap(array, order, range, 3, 7)
                    this.netSwap(array, order, range, 1, 5)
                    this.netSwap(array, order, range, 2, 6)
                    this.netSwap(array, order, range, 1, 4)
                    this.netSwap(array, order, range, 3, 6)
                    this.netSwap(array, order, range, 2, 4)
                    this.netSwap(array, order, range, 3, 5)
                    this.netSwap(array, order, range, 3, 4)
                case 7:
                    this.netSwap(array, order, range, 1, 2)
                    this.netSwap(array, order, range, 3, 4)
                    this.netSwap(array, order, range, 5, 6)
                    this.netSwap(array, order, range, 0, 2)
                    this.netSwap(array, order, range, 3, 5)
                    this.netSwap(array, order, range, 4, 6)
                    this.netSwap(array, order, range, 0, 1)
                    this.netSwap(array, order, range, 4, 5)
                    this.netSwap(array, order, range, 2, 6)
                    this.netSwap(array, order, range, 0, 4)
                    this.netSwap(array, order, range, 1, 5)
                    this.netSwap(array, order, range, 0, 3)
                    this.netSwap(array, order, range, 2, 5)
                    this.netSwap(array, order, range, 1, 3)
                    this.netSwap(array, order, range, 2, 4)
                    this.netSwap(array, order, range, 2, 3)
                case 6:
                    this.netSwap(array, order, range, 1, 2)
                    this.netSwap(array, order, range, 4, 5)
                    this.netSwap(array, order, range, 0, 2)
                    this.netSwap(array, order, range, 3, 5)
                    this.netSwap(array, order, range, 0, 1)
                    this.netSwap(array, order, range, 3, 4)
                    this.netSwap(array, order, range, 2, 5)
                    this.netSwap(array, order, range, 0, 3)
                    this.netSwap(array, order, range, 1, 4)
                    this.netSwap(array, order, range, 2, 4)
                    this.netSwap(array, order, range, 1, 3)
                    this.netSwap(array, order, range, 2, 3)
                case 5:
                    this.netSwap(array, order, range, 0, 1)
                    this.netSwap(array, order, range, 3, 4)
                    this.netSwap(array, order, range, 2, 4)
                    this.netSwap(array, order, range, 2, 3)
                    this.netSwap(array, order, range, 1, 4)
                    this.netSwap(array, order, range, 0, 3)
                    this.netSwap(array, order, range, 0, 2)
                    this.netSwap(array, order, range, 1, 3)
                    this.netSwap(array, order, range, 1, 2)
                case 4:
                    this.netSwap(array, order, range, 0, 1)
                    this.netSwap(array, order, range, 2, 3)
                    this.netSwap(array, order, range, 0, 2)
                    this.netSwap(array, order, range, 1, 3)
                    this.netSwap(array, order, range, 1, 2)
        if size < 8:
            return
        buffer1: WikiRange
        buffer2: WikiRange
        blockA: WikiRange
        blockB: WikiRange
        lastA: WikiRange
        lastB: WikiRange
        firstA: WikiRange
        A: WikiRange
        B: WikiRange
        buffer1 = WikiRange(0, 0)
        buffer2 = WikiRange(0, 0)
        blockA = WikiRange(0, 0)
        blockB = WikiRange(0, 0)
        lastA = WikiRange(0, 0)
        lastB = WikiRange(0, 0)
        firstA = WikiRange(0, 0)
        A = WikiRange(0, 0)
        B = WikiRange(0, 0)
        pull: list
        pull = [WikiPull(), WikiPull()]
        while True:
            if iterator.length() < this.cache_size:
                if (iterator.length()+1)*4 <= this.cache_size and iterator.length()*4 <= size:
                    iterator.begin()
                    while not iterator.finished():
                        A1: WikiRange
                        B1: WikiRange
                        A2: WikiRange
                        B2: WikiRange
                        A1 = iterator.nextRange()
                        B1 = iterator.nextRange()
                        A2 = iterator.nextRange()
                        B2 = iterator.nextRange()
                        if array[B1.end-1] < array[A1.start]:
                            arrayCopy(array, A1.start, this.cache,
                                      B1.length(), A1.length())
                            arrayCopy(array, B1.start,
                                      this.cache, 0, B1.length())
                        elif array[B1.start] < array[A1.end-1]:
                            this.mergeInto(array, A1, B1, this.cache, 0)
                        else:
                            if array[B2.start] >= array[A2.end-1] and array[A2.start] >= array[B1.end-1]:
                                continue
                            arrayCopy(array, A1.start,
                                      this.cache, 0, A1.length())
                            arrayCopy(array, B1.start, this.cache,
                                      A1.length(), B1.length())
                        A1.set(A1.start, B1.end)
                        if array[B2.end-1] < array[A2.start]:
                            arrayCopy(array, A2.start, this.cache,
                                      A1.length()+B2.length(), A2.length())
                            arrayCopy(array, B2.start, this.cache,
                                      A1.length(), B2.length())
                        elif array[B2.start] < array[A2.end-1]:
                            this.mergeInto(
                                array, A2, B2, this.cache, A1.length())
                        else:
                            arrayCopy(array, A2.start, this.cache,
                                      A1.length(), A2.length())
                            arrayCopy(array, B2.start, this.cache,
                                      A1.length()+A2.length(), B2.length())
                        A2.set(A2.start, B2.end)
                        A3: WikiRange
                        B3: WikiRange
                        A3 = WikiRange(0, A1.length())
                        B3 = WikiRange(A1.length(), A1.length()+A2.length())
                        if this.cache[B3.end-1] < this.cache[A3.start]:
                            arrayCopy(this.cache, A3.start, array,
                                      A1.start+A2.length(), A3.length())
                            arrayCopy(this.cache, B3.start, array,
                                      A1.start, B3.length())
                        elif this.cache[B3.start] < this.cache[A3.end-1]:
                            this.mergeInto(this.cache, A3, B3, array, A1.start)
                        else:
                            arrayCopy(this.cache, A3.start, array,
                                      A1.start, A3.length())
                            arrayCopy(this.cache, B3.start, array,
                                      A1.start+A1.length(), B3.length())
                    iterator.nextLevel()
                else:
                    iterator.begin()
                    while not iterator.finished():
                        A = iterator.nextRange()
                        B = iterator.nextRange()
                        if array[B.end-1] < array[A.start]:
                            this.rotate(array, A.length(),
                                        WikiRange(A.start, B.end), True)
                        elif array[B.start] < array[A.end-1]:
                            arrayCopy(array, A.start,
                                      this.cache, 0, A.length())
                            this.mergeExternal(array, A, B)
            else:
                block_size: int
                buffer_size: int
                index: int
                last: int
                count: int
                pull_index = 0
                block_size = int(math.sqrt(iterator.length()))
                buffer_size = iterator.length()//block_size+1
                buffer1.set(0, 0)
                buffer2.set(0, 0)
                pull[0].reset()
                pull[1].reset()
                find = buffer_size+buffer_size
                find_separately = False
                if block_size <= this.cache_size:
                    find = buffer_size
                elif find > iterator.length():
                    find = buffer_size
                    find_separately = True
                iterator.begin()
                while not iterator.finished():
                    A = iterator.nextRange()
                    B = iterator.nextRange()
                    last = A.start
                    count = 1
                    while count < find:
                        index = this.findLastForward(
                            array, array[last].readInt(), WikiRange(last+1, A.end), find-count)
                        if index == A.end:
                            break
                        last = index
                        count += 1
                    index = last
                    if count >= buffer_size:
                        pull[pull_index].range.set(A.start, B.end)
                        pull[pull_index].count = count
                        pull[pull_index].from_ = index
                        pull[pull_index].to = A.start
                        pull_index = 1
                        if count == buffer_size+buffer_size:
                            buffer1.set(A.start, A.start+buffer_size)
                            buffer2.set(A.start+buffer_size, A.start+count)
                            break
                        elif find == buffer_size+buffer_size:
                            buffer1.set(A.start, A.start+count)
                            find = buffer_size
                        elif block_size <= this.cache_size:
                            buffer1.set(A.start, A.start+count)
                            break
                        elif find_separately:
                            buffer1 = WikiRange(A.start, A.start+count)
                            find_separately = False
                        else:
                            buffer2.set(A.start, A.start+count)
                            break
                    elif pull_index == 0 and count > buffer1.length():
                        buffer1.set(A.start, A.start+count)
                        pull[pull_index].range.set(A.start, B.end)
                        pull[pull_index].count = count
                        pull[pull_index].from_ = index
                        pull[pull_index].to = A.start
                    last = B.end-1
                    count = 1
                    while count < find:
                        index = this.findFirstBackward(
                            array, array[last].readInt(), WikiRange(B.start, last), find-count)
                        if index == B.start:
                            break
                        last = index-1
                        count += 1
                    index = last
                    if count >= buffer_size:
                        pull[pull_index].range.set(A.start, B.end)
                        pull[pull_index].count = count
                        pull[pull_index].from_ = index
                        pull[pull_index].to = B.end
                        pull_index = 1
                        if count == buffer_size+buffer_size:
                            buffer1.set(B.end-count, B.end-buffer_size)
                            buffer2.set(B.end-buffer_size, B.end)
                            break
                        elif find == buffer_size+buffer_size:
                            buffer1.set(B.end-count, B.end)
                            find = buffer_size
                        elif block_size <= this.cache_size:
                            buffer1.set(B.end-count, B.end)
                            break
                        elif find_separately:
                            buffer1 = WikiRange(B.end-count, B.end)
                            find_separately = False
                        else:
                            if pull[0].range.start == A.start:
                                pull[0].range.end -= pull[1].count
                            buffer2.set(B.end-count, B.end)
                            break
                    elif pull_index == 0 and count > buffer1.length():
                        buffer1.set(B.end-count, B.end)
                        pull[pull_index].range.set(A.start, B.end)
                        pull[pull_index].count = count
                        pull[pull_index].from_ = index
                        pull[pull_index].to = B.end
                pull_index = 0
                while pull_index < 2:
                    length = pull[pull_index].count
                    if pull[pull_index].to < pull[pull_index].from_:
                        index = pull[pull_index].from_
                        count = 1
                        while count < length:
                            index = this.findFirstBackward(array, array[index-1].readInt(), WikiRange(
                                pull[pull_index].to, pull[pull_index].from_-(count-1)), length-count)
                            range: WikiRange
                            range = WikiRange(
                                index+1, pull[pull_index].from_+1)
                            this.rotate(array, range.length() -
                                        count, range, True)
                            pull[pull_index].from_ = index+count
                            count += 1
                    elif pull[pull_index].to > pull[pull_index].from_:
                        index = pull[pull_index].from_+1
                        count = 1
                        while count < length:
                            index = this.findLastForward(array, array[index].readInt(
                            ), WikiRange(index, pull[pull_index].to), length-count)
                            range: WikiRange
                            range = WikiRange(pull[pull_index].from_, index-1)
                            this.rotate(array, count, range, True)
                            pull[pull_index].from_ = index-1-count
                            count += 1
                    pull_index += 1
                buffer_size = buffer1.length()
                block_size = iterator.length()//buffer_size+1
                iterator.begin()
                while not iterator.finished():
                    A = iterator.nextRange()
                    B = iterator.nextRange()
                    start = A.start
                    if start == pull[0].range.start:
                        if pull[0].from_ > pull[0].to:
                            A.start += pull[0].count
                            if A.length() == 0:
                                continue
                        elif pull[0].from_ < pull[0].to:
                            B.end -= pull[0].count
                            if B.length() == 0:
                                continue
                    if start == pull[1].range.start:
                        if pull[1].from_ > pull[1].to:
                            A.start += pull[1].count
                            if A.length() == 0:
                                continue
                        elif pull[1].from_ < pull[1].to:
                            B.end -= pull[1].count
                            if B.length() == 0:
                                continue
                    if array[B.end-1] < array[A.start]:
                        this.rotate(array, A.length(),
                                    WikiRange(A.start, B.end), True)
                    elif array[A.end] < array[A.end-1]:
                        blockA.set(A.start, A.end)
                        firstA.set(A.start, A.start+blockA.length() %
                                   block_size)
                        indexA = buffer1.start
                        index = firstA.end
                        while index < blockA.end:
                            array[indexA].swap(array[index])
                            indexA += 1
                            index += block_size
                        lastA.set(firstA.start, firstA.end)
                        lastB.set(0, 0)
                        blockB.set(B.start, B.start +
                                   min(block_size, B.length()))
                        blockA.start += firstA.length()
                        indexA = buffer1.start
                        if lastA.length() <= this.cache_size and this.cache is not None:
                            arrayCopy(array, lastA.start,
                                      this.cache, 0, lastA.length())
                        elif buffer2.length() > 0:
                            blockSwap(array, lastA.start,
                                      buffer2.start, lastA.length())
                        if blockA.length() > 0:
                            while True:
                                if (lastB.length() > 0 and array[lastB.end-1] >= array[indexA]) or blockB.length() == 0:
                                    B_split: int
                                    B_remaining: int
                                    minA: int
                                    B_split = this.binaryFirst(
                                        array, array[indexA].readInt(), lastB)
                                    B_remaining = lastB.end-B_split
                                    minA = blockA.start
                                    findA = minA+block_size
                                    while findA < blockA.end:
                                        if array[findA] < array[minA]:
                                            minA = findA
                                        findA += block_size
                                    blockSwap(array, blockA.start,
                                              minA, block_size)
                                    array[blockA.start].swap(array[indexA])
                                    indexA += 1
                                    if lastA.length() <= this.cache_size:
                                        this.mergeExternal(
                                            array, lastA, WikiRange(lastA.end, B_split))
                                    elif buffer2.length() > 0:
                                        this.mergeInternal(array, lastA, WikiRange(
                                            lastA.end, B_split), buffer2)
                                    else:
                                        this.mergeInPlace(
                                            array, lastA, WikiRange(lastA.end, B_split))
                                    if buffer2.length() > 0 or block_size <= this.cache_size:
                                        if block_size <= this.cache_size:
                                            arrayCopy(
                                                array, blockA.start, this.cache, 0, block_size)
                                        else:
                                            blockSwap(
                                                array, blockA.start, buffer2.start, block_size)
                                        blockSwap(
                                            array, B_split, blockA.start+block_size-B_remaining, B_remaining)
                                    else:
                                        this.rotate(
                                            array, blockA.start-B_split, WikiRange(B_split, blockA.start+block_size), True)
                                    lastA.set(blockA.start-B_remaining,
                                              blockA.start-B_remaining+block_size)
                                    lastB.set(lastA.end, lastA.end+B_remaining)
                                    blockA.start += block_size
                                    if blockA.length() == 0:
                                        break
                                elif blockB.length() < block_size:
                                    this.rotate(
                                        array, -blockB.length(), WikiRange(blockA.start, blockB.end), False)
                                    lastB.set(blockA.start,
                                              blockA.start+blockB.length())
                                    blockA.start += blockB.length()
                                    blockA.end += blockB.length()
                                    blockB.end = blockB.start
                                else:
                                    blockSwap(array, blockA.start,
                                              blockB.start, block_size)
                                    lastB.set(blockA.start,
                                              blockA.start+block_size)
                                    blockA.start += block_size
                                    blockA.end += block_size
                                    blockB.start += block_size
                                    blockB.end += block_size
                                    if blockB.end > B.end:
                                        blockB.end = B.end
                        if lastA.length() <= this.cache_size:
                            this.mergeExternal(
                                array, lastA, WikiRange(lastA.end, B.end))
                        elif buffer2.length() > 0:
                            this.mergeInternal(array, lastA, WikiRange(
                                lastA.end, B.end), buffer2)
                        else:
                            this.mergeInPlace(
                                array, lastA, WikiRange(lastA.end, B.end))
                this.insertionSort(array, buffer2)
                pull_index = 0
                while pull_index < 2:
                    unique = pull[pull_index].count*2
                    if pull[pull_index].from_ > pull[pull_index].to:
                        buffer: WikiRange
                        buffer = WikiRange(
                            pull[pull_index].range.start, pull[pull_index].range.start+pull[pull_index].count)
                        while buffer.length() > 0:
                            index = this.findFirstForward(array, array[buffer.start].readInt(
                            ), WikiRange(buffer.end, pull[pull_index].range.end), unique)
                            amount = index-buffer.end
                            this.rotate(array, buffer.length(),
                                        WikiRange(buffer.start, index), True)
                            buffer.start += (amount+1)
                            buffer.end += amount
                            unique -= 2
                    elif pull[pull_index].from_ < pull[pull_index].to:
                        buffer: WikiRange
                        buffer = WikiRange(
                            pull[pull_index].range.end-pull[pull_index].count, pull[pull_index].range.end)
                        while buffer.length() > 0:
                            index = this.findLastBackward(
                                array, array[buffer.end-1], WikiRange(pull[pull_index].range.start, buffer.start), unique)
                            amount = buffer.start-index
                            this.rotate(array, amount, WikiRange(
                                index, buffer.end), True)
                            buffer.start -= amount
                            buffer.end -= (amount+1)
                            unique -= 2
                    pull_index += 1
            if not iterator.nextLevel():
                break


@Sort("Block Merge Sorts", "Wiki Sort", "Wiki Sort")
def wikiSortRun(array):
    mode = sortingVisualizer.getUserInput(
        "Insert buffer size (0 for in-place)", "0", parseInt)
    WikiSort(mode, None, None).sort(array, len(array))
