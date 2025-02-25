from abc import ABC, abstractmethod
from itertools import chain
from time import time
from typing import List

import numpy as np
import pandas as pd


class Timer:
    def __enter__(self):
        self.start = time()
        return self

    def __exit__(self, *args):
        self.end = time()
        self.interval = self.end - self.start


class RangeInterface(ABC):
    __slots__ = ['_begin', '_end']

    def __init__(self, begin, end):
        self._begin = begin
        self._end = end

    @property
    def begin(self) -> int:
        return self._begin

    @begin.setter
    def begin(self, new_val):
        self._begin = new_val

    @property
    def end(self) -> int:
        return self._end

    @end.setter
    def end(self, new_val):
        self._end = new_val

    @staticmethod
    def min():
        return 0

    @staticmethod
    @abstractmethod
    def max() -> int:
        pass

    def __str__(self):
        return f'({self.begin}, {self.end})'


class RangeIP(RangeInterface):
    __max = pow(2, 32) - 1

    @staticmethod
    def max():
        return RangeIP.__max


class RangePort(RangeInterface):
    __max = pow(2, 16) - 1

    @staticmethod
    def max():
        return RangePort.__max


class RangeIPPort(RangeInterface):
    __max = pow(2, 32 + 16) - 1

    @staticmethod
    def max():
        return RangeIPPort.__max


class LogicalOperations:  # сделать миксину
    __slots__ = ['ranges', 'type_']
    ranges: List[RangeInterface]
    type_: RangeInterface

    def __init__(self, ranges, type_=None):
        self.type_ = type_

        if type(ranges) is not list:
            raise NotImplementedError

        if len(ranges) > 0:
            my_type_ = type(ranges[0])
            if not issubclass(my_type_, RangeInterface):
                raise NotImplementedError

            if self.type_ is None:
                self.type_ = my_type_

        if self.type_ is None:
            raise NotImplementedError

        self.ranges = ranges

    def __invert__(self):
        ans = []
        type_ = self.type_ or type(self.ranges[0])

        if self.ranges[0].begin != 0:
            ans.append(type_(0, self.ranges[0].begin - 1))

        for i, j in zip(self.ranges[:-1], self.ranges[1:]):
            begin = i.end + 1
            end = j.begin - 1

            if begin <= end:
                ans.append(type_(begin, end))

        max_ip_num = self.ranges[-1].max()
        if self.ranges[-1].end != max_ip_num:
            ans.append(type_(self.ranges[-1].end + 1, max_ip_num))

        return LogicalOperations(ans, type_=type_)

    def __or__(self, other):
        if isinstance(other, LogicalOperations):
            intervals = sorted(chain(self.ranges, other.ranges), key=lambda x: x.begin)
            type_ = self.type_ or type(self.ranges[0])

            ans = [intervals[0]]

            for i in intervals[1:]:
                if ans[-1].begin <= i.begin <= ans[-1].end or ans[-1].end + 1 == i.end:
                    end = max(ans[-1].end, i.end)
                    ans[-1].end = end
                else:
                    ans.append(i)

            return LogicalOperations(ans, type_=type_)
        else:
            return NotImplemented

    def __and__(self, other):
        if isinstance(other, LogicalOperations):
            ans = []
            type_ = self.type_ or type(self.ranges[0])

            self_iter = 0
            other_iter = 0

            while self_iter < len(self.ranges) and other_iter < len(other.ranges):
                if other.ranges[other_iter].begin <= self.ranges[self_iter].end and \
                   self.ranges[self_iter].begin <= other.ranges[other_iter].end:

                    left = max(self.ranges[self_iter].begin, other.ranges[other_iter].begin)
                    right = min(self.ranges[self_iter].end, other.ranges[other_iter].end)

                    ans.append(type_(left, right))

                if self.ranges[self_iter].end > other.ranges[other_iter].end:
                    other_iter += 1
                else:
                    self_iter += 1

            return LogicalOperations(ans, type_=type_)
        else:
            return NotImplemented

    def __sub__(self, other):
        if isinstance(other, LogicalOperations):
            return self & ~other
        else:
            return NotImplemented

    def __xor__(self, other):
        if isinstance(other, LogicalOperations):
            return self - other | other - self
        else:
            return NotImplemented

    def __str__(self):
        return f"[{', '.join([str(x) for x in self.ranges])}]"


class LogicalOperationsNP:
    max_value = pow(2, 32 + 16) - 1

    __slots__ = ['ranges']
    ranges: List[RangeInterface]

    @staticmethod
    def min():
        return 0

    @staticmethod
    def max():
        return LogicalOperationsNP.max_value

    def __init__(self, ranges):
        self.ranges = ranges

    def __invert__(self):
        if len(self.ranges) == 0:
            return LogicalOperationsNP(np.array([self.min(), self.max()], dtype='int64').reshape((-1, 2)))

        ranges = self.ranges.copy()
        append_to_front = int(ranges[0, 0] == self.min())
        append_to_back = int(ranges[-1, 1] == self.max())

        ranges[:, 0] -= 1
        ranges[:, 1] += 1

        ranges = ranges.reshape(1, -1)[0]
        ranges = ranges[append_to_front:ranges.size - append_to_back]

        first_element = None if append_to_front == 1 else self.min()
        last_element = None if append_to_back == 1 else self.max()
        ranges = np.concatenate([x for x in ((first_element,), ranges, (last_element,)) if None not in x])
        ranges = ranges.reshape((-1, 2))

        return LogicalOperationsNP(ranges)

    def __or__(self, other):
        if isinstance(other, LogicalOperationsNP):
            intervals = np.concatenate([x for x in (self.ranges, other.ranges)], axis=0)
            intervals.sort(axis=0)

            intervals = pd.DataFrame(intervals, columns=['start', 'end'])
            intervals["merge_intervals"] = (intervals['start'] == (intervals['end'].shift() + 1).cummax())
            intervals["group"] = ((intervals["start"] > intervals["end"].shift()) ^ intervals["merge_intervals"]).cumsum()

            ans = intervals.groupby("group").agg({"start": "min", "end": "max"})

            return LogicalOperationsNP(ans.values)
        else:
            return NotImplemented

    def __and__(self, other):
        if isinstance(other, LogicalOperationsNP):
            intervals = np.concatenate([x for x in (self.ranges, other.ranges)], axis=0)
            intervals.sort(axis=0)

            intervals = pd.DataFrame(intervals, columns=['start', 'end'])
            intervals['intersects'] = (intervals.end + 1 - intervals.start.shift(-1)) > 0
            intervals['start'] = intervals['start'].shift(-1)

            ans = intervals[(intervals['intersects'] == True)][['start', 'end']] # noqa

            return LogicalOperationsNP(ans.values.astype('int64'))
        else:
            return NotImplemented

    def __sub__(self, other):
        if isinstance(other, LogicalOperationsNP):
            return self & ~other
        else:
            return NotImplemented

    def __xor__(self, other):
        if isinstance(other, LogicalOperationsNP):
            return self - other | other - self
        else:
            return NotImplemented

    def __str__(self):
        ranges = []
        for i in range(0, self.ranges.shape[0]):
            ranges.append(f'({self.ranges[i, 0]}, {self.ranges[i, 1]})')

        return f"[{', '.join([str(x) for x in ranges])}]"


ports_ranges_1 = [
    RangeIPPort(begin=0, end=1000),
    RangeIPPort(begin=1500, end=2000),
]

ports_ranges_2 = [
    RangeIPPort(begin=0, end=1000),
    RangeIPPort(begin=1100, end=2000),
]

ranges_1 = LogicalOperations(ports_ranges_1)

print('invert')
print(ranges_1)
print(~ranges_1)
print()

ranges_2 = LogicalOperations(ports_ranges_2)
print('logical and')
print(ranges_1)
print(ranges_2)
print(ranges_1 & ranges_2)
print()

print('logical or')
print(ranges_1)
print(ranges_2)
print(ranges_1 | ranges_2)
print()

print('logical sub var 1')
print(ranges_1)
print(ranges_2)
print(ranges_1 - ranges_2)
print()

print('logical sub var 2')
print(ranges_1)
print(ranges_2)
print(ranges_2 - ranges_1)
print()

print('logical xor')
print(ranges_1)
print(ranges_2)
print(ranges_1 ^ ranges_2)
print()


class Mask:
    __slots__ = ['ips_ranges', 'ports_ranges']
    ips_ranges: List[RangeIP]
    ports_ranges: List[RangePort]

    def __init__(self, ips_ranges, ports_ranges):
        self.ips_ranges = ips_ranges
        self.ports_ranges = ports_ranges

    def to_collapsed(self) -> LogicalOperations:
        ans = []
        for addr in self.ips_ranges:
            for ip in range(addr.begin, addr.end + 1):
                for port_range in self.ports_ranges:
                    ans.append(RangeIPPort(
                        int.from_bytes(ip.to_bytes(4, 'big') + port_range.begin.to_bytes(2, 'big'), 'big'),
                        int.from_bytes(ip.to_bytes(4, 'big') + port_range.end.to_bytes(2, 'big'), 'big')
                    ))

        return LogicalOperations(ans)

    @staticmethod
    def from_collapsed(ranges: LogicalOperations) -> 'Mask':
        pass

    def to_collapsed_numpy(self) -> LogicalOperationsNP:
        ans = np.array([], dtype='int64').reshape((-1, 2))

        for addr in self.ips_ranges:
            base_arr = np.arange(start=addr.begin, stop=addr.end + 1, dtype='int64')
            base_arr = base_arr << 16

            for port_range in self.ports_ranges:
                arr_starts = base_arr + port_range.begin
                arr_ends = base_arr + port_range.end

                ans = np.concatenate((ans, np.array([arr_starts, arr_ends]).T), axis=0)

        # минимизация, чтобы избавиться от некорректных данных, которые могут прийти в класс
        # так как нигде ранее не стоит проверки на корректность последовательности
        ans = ans & ans

        return LogicalOperationsNP(ans)

    @staticmethod
    def from_collapsed_numpy(ranges: LogicalOperationsNP) -> 'Mask':
        pass


def equals_tests():
    mask_1 = Mask(
        ips_ranges=[
            RangeIP(0, 0)
        ],

        ports_ranges=[
            RangePort(0, 1000),
            RangePort(1500, 2000)
        ]
    )

    mask_2 = Mask(
        ips_ranges=[
            RangeIP(0, 0)
        ],

        ports_ranges=[
            RangePort(0, 1000),
            RangePort(1100, 2000)
        ]
    )

    print('collapsed mask 1')
    print(mask_1.to_collapsed())
    print(mask_1.to_collapsed_numpy())
    print()

    print('collapsed mask 2')
    print(mask_2.to_collapsed())
    print(mask_2.to_collapsed_numpy())
    print()

    print('invert mask 1')
    print(~mask_1.to_collapsed())
    print(~mask_1.to_collapsed_numpy())
    print()

    print('invert mask 2')
    print(~mask_2.to_collapsed())
    print(~mask_2.to_collapsed_numpy())
    print()

    print('or masks')
    print(mask_1.to_collapsed() | mask_2.to_collapsed())
    print(mask_1.to_collapsed_numpy() | mask_2.to_collapsed_numpy())
    print()

    print('and masks')
    print(mask_1.to_collapsed() & mask_2.to_collapsed())
    print(mask_1.to_collapsed_numpy() & mask_2.to_collapsed_numpy())
    print()


def performance_tests(ip_addrs_power):
    count = 1 << ip_addrs_power

    rules_1 = Mask(
        ips_ranges=[RangeIP(0, count)],
        ports_ranges=[RangePort(0, 1500)],
    )

    rules_2 = Mask(
        ips_ranges=[RangeIP(0, count)],
        ports_ranges=[RangePort(0, 1000)],
    )

    print(f'Тест производительности для {count} диапазонов в двух множествах')

    with Timer() as old_logic_or_t:
        _ = rules_1.to_collapsed() | rules_2.to_collapsed()
    with Timer() as new_logic_or_t:
        _ = rules_1.to_collapsed_numpy() | rules_2.to_collapsed_numpy()
    print(f'Логическое или (алг): {old_logic_or_t.interval}, Векторизованный алгоритм: {new_logic_or_t.interval}')

    with Timer() as old_logic_and_t:
        _ = rules_1.to_collapsed() & rules_2.to_collapsed()
    with Timer() as new_logic_and_t:
        _ = rules_1.to_collapsed_numpy() & rules_2.to_collapsed_numpy()
    print(f'Логическое и (алг): {old_logic_and_t.interval}, Векторизованный алгоритм: {new_logic_and_t.interval}')

    with Timer() as old_logic_xor_t:
        _ = rules_1.to_collapsed() ^ rules_2.to_collapsed()
    with Timer() as new_logic_xor_t:
        _ = rules_1.to_collapsed_numpy() ^ rules_2.to_collapsed_numpy()
    print(f'Исключающие или (алг): {old_logic_xor_t.interval}, Векторизованный алгоритм: {new_logic_xor_t.interval}')

    with Timer() as old_sub_var_1_t:
        _ = rules_1.to_collapsed() - rules_2.to_collapsed()
    with Timer() as new_sub_var_1_t:
        _ = rules_1.to_collapsed_numpy() - rules_2.to_collapsed_numpy()
    print(f'Логическое вычитание. Вариант 1 (алг): {old_sub_var_1_t.interval}, Векторизованный алгоритм: {new_sub_var_1_t.interval}')

    with Timer() as old_sub_var_2_t:
        _ = rules_2.to_collapsed() - rules_1.to_collapsed()
    with Timer() as new_sub_var_2_t:
        _ = rules_2.to_collapsed_numpy() - rules_1.to_collapsed_numpy()
    print(f'Логическое вычитание. Вариант 2 (алг): {old_sub_var_2_t.interval}, Векторизованный алгоритм: {new_sub_var_2_t.interval}')

    statistics = f'' \
    f'{count}\t' \
    f'{old_logic_or_t.interval: .5f}\t' \
    f'{new_logic_or_t.interval: .5f}\t' \
    f'{old_logic_and_t.interval: .5f}\t' \
    f'{new_logic_and_t.interval: .5f}\t' \
    f'{old_logic_xor_t.interval: .5f}\t' \
    f'{new_logic_xor_t.interval: .5f}\t' \
    f'{old_sub_var_1_t.interval: .5f}\t' \
    f'{new_sub_var_1_t.interval: .5f}\t' \
    f'{old_sub_var_2_t.interval: .5f}\t' \
    f'{new_sub_var_2_t.interval: .5f}'

    return statistics


def performance_tests_numpy_only(ip_addrs_power):
    count = 1 << ip_addrs_power

    rules_1 = Mask(
        ips_ranges=[RangeIP(0, count)],
        ports_ranges=[RangePort(0, 1500)],
    ).to_collapsed_numpy()

    rules_2 = Mask(
        ips_ranges=[RangeIP(0, count)],
        ports_ranges=[RangePort(0, 1000)],
    ).to_collapsed_numpy()

    print(f'Тест производительности для {count} диапазонов в двух множествах')

    with Timer() as new_logic_or_t:
        _ = rules_1 | rules_2
    print(f'Логическое ИЛИ. Только NumPy: {new_logic_or_t.interval}')

    with Timer() as new_logic_and_t:
        _ = rules_1 & rules_2
    print(f'Логическое И. Только NumPy: {new_logic_and_t.interval}')

    with Timer() as new_logic_xor_t:
        _ = rules_1 ^ rules_2
    print(f'Исключающие ИЛИ. Только NumPy: {new_logic_xor_t.interval}')

    with Timer() as new_sub_var_1_t:
        _ = rules_1 - rules_2
    print(f'Логическое вычитание. Только NumPy: {new_sub_var_1_t.interval}')

    with Timer() as new_sub_var_2_t:
        _ = rules_2 - rules_1
    print(f'Логическое вычитание. Только NumPy: {new_sub_var_2_t.interval}')

    statistics = f'' \
    f'{count}\t' \
    f'{new_logic_or_t.interval: .5f}\t' \
    f'{new_logic_and_t.interval: .5f}\t' \
    f'{new_logic_xor_t.interval: .5f}\t' \
    f'{new_sub_var_1_t.interval: .5f}\t' \
    f'{new_sub_var_2_t.interval: .5f}'

    return statistics


if __name__ == '__main__':
    equals_tests()

    statistics = []
    for i in range(21):
        statistics.append(performance_tests(i))

    headers = \
        f'' \
        f'Размер входных данных\t' \
        f'Логическое или (PyPy)\t' \
        f'Логическое и (PyPy)\t' \
        f'Исключающие или (PyPy)\t' \
        f'Вычитание вариант 1 (PyPy)\t' \
        f'Вычитание вариант 2 (PyPy)\t'

    print(headers)
    print('\n'.join([str(x).replace('.', ',') for x in statistics]))

    statistics = []
    for i in range(21):
        statistics.append(performance_tests_numpy_only(i))

    headers = \
        f'' \
        f'Размер входных данных\t' \
        f'Логическое или (Только NumPy)\t' \
        f'Логическое и (Только NumPy)\t' \
        f'Исключающие или (Только NumPy)\t' \
        f'Вычитание вариант 1 (Только NumPy)\t' \
        f'Вычитание вариант 2 (Только NumPy)\t'

    print(headers)
    print('\n'.join([str(x).replace('.', ',') for x in statistics]))
