#include <iostream>
#include <utility>
#include <limits>
#include <vector>
#include <algorithm>
#include <chrono>
#include <iomanip>
#include <sstream>


using namespace std;
using namespace chrono;

template <typename T>
class Range {
public:
    Range(T begin, T end): begin(begin), end(end) {};

    static T min() {
        return numeric_limits<T>::min();
    }

    static T max() {
        return numeric_limits<T>::max();
    }

    T begin;
    T end;
};

template <typename T>
ostream& operator << (ostream &os, const Range<T>& printed) {
    return os << '(' << printed.begin << ", " << printed.end << ')';
}


typedef Range<uint16_t> PortRange;
typedef Range<uint32_t> IPRange;
typedef Range<uint64_t> IPPortRange;


template <typename T>
class LogicalOperations {
public:
    explicit LogicalOperations(vector<T> && ranges) : ranges(std::move(ranges)) {};

    LogicalOperations<T> operator and(const LogicalOperations<T>& other_ranges) const {
        auto own_index = 0;
        auto other_index = 0;

        vector<T> ans;
        while(own_index < ranges.size() and other_index < other_ranges.ranges.size()) {
            if(other_ranges.ranges[other_index].begin <= ranges[own_index].end and
               ranges[own_index].begin <= other_ranges.ranges[other_index].end) {
                auto left = max(ranges[own_index].begin, other_ranges.ranges[other_index].begin);
                auto right = min(ranges[own_index].end, other_ranges.ranges[other_index].end);

                ans.emplace_back(T(left, right));
            }

            if(ranges[own_index].end > other_ranges.ranges[other_index].end) {
                other_index++;
            } else {
                own_index++;
            }
        }

        return std::move(LogicalOperations<T>(std::move(ans)));
    }

    LogicalOperations<T> operator or (const LogicalOperations<T>& other_ranges) const {
        vector<T> all_ranges;
        all_ranges.reserve(ranges.size() + other_ranges.ranges.size());

        all_ranges.insert(all_ranges.end(), ranges.begin(), ranges.end());
        all_ranges.insert(all_ranges.end(), other_ranges.ranges.begin(), other_ranges.ranges.end());

        sort(all_ranges.begin(), all_ranges.end(), [](T const & a, T const & b) { return a.begin < b.begin; });

        vector<T> ans;
        ans.reserve(all_ranges.size());
        ans.emplace_back(all_ranges.front());
        for(auto i = 1; i < all_ranges.size(); ++i) {
            if(ans.back().end + 1 >= all_ranges[i].begin and ans.back().end <= all_ranges[i].end) {
                ans.back().end = max(ans.back().end, all_ranges[i].end);
            } else {
                ans.emplace_back(all_ranges[i]);
            }
        }

        return std::move(LogicalOperations<T>(std::move(ans)));
    }

    LogicalOperations<T> operator!() const {
        if(ranges.size() == 0) {
            return LogicalOperations<T>({T(T::min(), T::max())});
        }

        vector<T> ans;
        ans.reserve(ranges.size() + 2);

        if(ranges.front().begin != 0) {
            ans.emplace_back(T(T::min(), ranges.front().begin - 1));
        }

        for(auto i = 0; i < ranges.size() - 1; ++i) {
            auto begin = ranges[i].end + 1;
            auto end = ranges[i + 1].begin - 1;

            ans.emplace_back(T(begin, end));
        }

        if(ranges.back().end != T::max()) {
           ans.emplace_back(T(ranges.back().end + 1, T::max()));
        }

        return std::move(LogicalOperations<T>(std::move(ans)));
    }

    LogicalOperations<T> operator-(const LogicalOperations<T>& other_ranges) const {
        return *this && !LogicalOperations<T>(other_ranges);
    }

    LogicalOperations<T> operator^(const LogicalOperations<T>& other_ranges) const {
        return *this - other_ranges || other_ranges - *this;
    }

    vector<T> ranges;
};


class Mask {
public:
    Mask(
        const vector<IPRange>& ips_ranges,
        const vector<PortRange>& port_ranges
    ):
        ips_ranges(ips_ranges),
        port_ranges(port_ranges) {
    }

    LogicalOperations<IPPortRange> to_collapsed() {
        vector<IPPortRange> ans;

        for(auto addrs :ips_ranges) {
            for(uint64_t ip = addrs.begin; ip <= addrs.end; ++ip) {
                uint64_t base_addr = ip << 16;

                for(auto port  :port_ranges) {
                    ans.emplace_back(base_addr + port.begin, base_addr + port.end);
                }
            }
        }

        return std::move(LogicalOperations<IPPortRange>(std::move(ans)));
    }

    vector<IPRange> ips_ranges;
    vector<PortRange> port_ranges;
};

string performance_test(uint ip_addrs_power) {
    uint count = 1 << ip_addrs_power;

    cout << "Тест производительности для " << count << " диапазонов в двух множествах" << endl;

    Mask m1(
        vector<IPRange>({
            IPRange(0, count),
        }),
        vector<PortRange>({
              PortRange(0, 1500),
        })
    );

    Mask m2(
    vector<IPRange>({
            IPRange(0, count),
        }),
vector<PortRange>({
            PortRange(0, 1000),
        })
    );

    cout << fixed << setprecision(4);

    auto prev_timer = high_resolution_clock::now();
    m1.to_collapsed() || m2.to_collapsed();
    auto old_logic_or_t = duration_cast<duration<float>>( high_resolution_clock::now() - prev_timer).count();
    cout << "Логическое или. С++: " << old_logic_or_t << endl;

    prev_timer = high_resolution_clock::now();
    m1.to_collapsed() && m2.to_collapsed();
    auto old_logic_and_t = duration_cast<duration<float>>(high_resolution_clock::now() - prev_timer).count();
    cout << "Логическое и. С++: " << old_logic_and_t << endl;

    prev_timer = high_resolution_clock::now();
    m1.to_collapsed() ^ m2.to_collapsed();
    auto old_logic_xor_t = duration_cast<duration<float>>(high_resolution_clock::now() - prev_timer).count();
    cout << "Исключающие или. С++: " << old_logic_xor_t << endl;

    prev_timer = high_resolution_clock::now();
    m1.to_collapsed() - m2.to_collapsed();
    auto old_logic_sub_var_1_t = duration_cast<duration<float>>(high_resolution_clock::now() - prev_timer).count();
    cout << "Логическое вычитание. Вариант 1. С++: " << old_logic_sub_var_1_t << endl;

    prev_timer = high_resolution_clock::now();
    m2.to_collapsed() - m1.to_collapsed();
    auto old_logic_sub_var_2_t = duration_cast<duration<float>>(high_resolution_clock::now() - prev_timer).count();
    cout << "Логическое вычитание. Вариант 2. С++: " << old_logic_sub_var_2_t << endl;

    stringstream result;

    result << count << '\t';
    result << fixed << std::setprecision(4);
    result << old_logic_or_t << '\t';
    result << old_logic_and_t << '\t';
    result << old_logic_xor_t << '\t';
    result << old_logic_sub_var_1_t << '\t';
    result << old_logic_sub_var_2_t << '\t';

    return result.str();
}

int main() {
    LogicalOperations ranges1(vector<PortRange>{
        PortRange(0, 1500)
    });

    LogicalOperations ranges2(vector<PortRange>{
        PortRange(0, 1000),
    });

    vector<string> results;
    for(auto i = 1; i < 21; ++i) {
        auto tmp_result = performance_test(i);
        replace(tmp_result.begin(), tmp_result.end(), '.', ',');
        results.push_back(tmp_result);
    }

    cout << "Размер входных данных" << '\t';
    cout << "Логическое или C++" << '\t';
    cout << "Логическое и C++" << '\t';
    cout << "Исключающие или C++" << '\t';
    cout << "Вычитание вариант 1 C++" << '\t';
    cout << "Вычитание вариант 2 C++" << '\t';
    cout << endl;

    for(auto & res: results) {
        cout << res << endl;
    }

    return 0;
}

