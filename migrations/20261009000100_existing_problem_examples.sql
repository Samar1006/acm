-- Tags and worked examples for the practice problems that already exist, matched
-- by title. Each example copies one of the problem's own visible tests, so its
-- expected output is the reference solution's. Databases without these
-- problems are left unchanged.

-- Integer Partitions
insert into problem_tags (problem_id, name) select id, 'Dynamic Programming' from problems where title = 'Integer Partitions'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Dynamic Programming');
insert into problem_tags (problem_id, name) select id, 'Math' from problems where title = 'Integer Partitions'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Math');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'There are two ways to write 2 as a sum of positive integers when order does not matter: 2 and 1 + 1.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Integer Partitions' and tests.test_number = 2 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'The seven partitions of 5 are 5, 4 + 1, 3 + 2, 3 + 1 + 1, 2 + 2 + 1, 2 + 1 + 1 + 1, and 1 + 1 + 1 + 1 + 1.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Integer Partitions' and tests.test_number = 3 and tests.hidden = false;

-- Count Primes Below N
insert into problem_tags (problem_id, name) select id, 'Math' from problems where title = 'Count Primes Below N'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Math');
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Count Primes Below N'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'The primes smaller than 10 are 2, 3, 5, and 7, so the answer is 4. The number 10 itself is not counted.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Count Primes Below N' and tests.test_number = 3 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'Only 2 is a prime smaller than 3. The number 3 is prime, but it is not strictly below 3.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Count Primes Below N' and tests.test_number = 2 and tests.hidden = false;

-- Sum of Multiples of 3 or 5
insert into problem_tags (problem_id, name) select id, 'Math' from problems where title = 'Sum of Multiples of 3 or 5'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Math');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'Below 10, the multiples of 3 or 5 are 3, 5, 6, and 9. Their sum is 3 + 5 + 6 + 9 = 23.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Sum of Multiples of 3 or 5' and tests.test_number = 3 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'Below 16, the multiples are 3, 5, 6, 9, 10, 12, and 15. The number 15 is a multiple of both but is added only once, giving 60.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Sum of Multiples of 3 or 5' and tests.test_number = 4 and tests.hidden = false;

-- N-Queens
insert into problem_tags (problem_id, name) select id, 'Backtracking' from problems where title = 'N-Queens'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Backtracking');
insert into problem_tags (problem_id, name) select id, 'Recursion' from problems where title = 'N-Queens'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Recursion');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'On a 4 by 4 board there are exactly two ways to place four queens so that no two share a row, column, or diagonal. Each board is a list of rows, where Q marks a queen.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'N-Queens' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'A 1 by 1 board has a single square, so the only arrangement puts the queen there.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'N-Queens' and tests.test_number = 1 and tests.hidden = false;

-- N-Queens II
insert into problem_tags (problem_id, name) select id, 'Backtracking' from problems where title = 'N-Queens II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Backtracking');
insert into problem_tags (problem_id, name) select id, 'Recursion' from problems where title = 'N-Queens II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Recursion');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'A 4 by 4 board has exactly two arrangements of four queens that do not attack each other. They are mirror images.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'N-Queens II' and tests.test_number = 3 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'With one square and one queen there is exactly one arrangement.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'N-Queens II' and tests.test_number = 0 and tests.hidden = false;

-- Best Time to Buy and Sell Stock III
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Best Time to Buy and Sell Stock III'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Dynamic Programming' from problems where title = 'Best Time to Buy and Sell Stock III'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Dynamic Programming');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'Buy at 0 and sell at 3 for a profit of 3, then buy at 1 and sell at 4 for another 3. Two trades give 6, and no other pair does better.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Best Time to Buy and Sell Stock III' and tests.test_number = 4 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'The price falls every day, so any purchase would lose money. Making no trade gives the best profit, 0.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Best Time to Buy and Sell Stock III' and tests.test_number = 1 and tests.hidden = false;

-- Coin Change
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Coin Change'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Dynamic Programming' from problems where title = 'Coin Change'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Dynamic Programming');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, '11 can be made with three coins: 5 + 5 + 1. No two coins from this set add up to 11.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Coin Change' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'Every total made from 2-value coins is even, so the odd amount 3 can never be reached. The answer is -1.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Coin Change' and tests.test_number = 1 and tests.hidden = false;

-- Permutations II
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Permutations II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Backtracking' from problems where title = 'Permutations II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Backtracking');
insert into problem_tags (problem_id, name) select id, 'Sorting' from problems where title = 'Permutations II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Sorting');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'The two 1s are identical, so swapping them does not create a new arrangement. Only three different orders remain.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Permutations II' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'All three values are equal, so every ordering looks the same and there is just one permutation.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Permutations II' and tests.test_number = 3 and tests.hidden = false;

-- Permutations
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Permutations'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Backtracking' from problems where title = 'Permutations'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Backtracking');
insert into problem_tags (problem_id, name) select id, 'Recursion' from problems where title = 'Permutations'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Recursion');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'Two different values can be ordered in two ways: 0 first or 1 first.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Permutations' and tests.test_number = 1 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'Three different values have 3 x 2 x 1 = 6 orderings. They are listed in sorted order.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Permutations' and tests.test_number = 0 and tests.hidden = false;

-- Single Number II
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Single Number II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Bit Manipulation' from problems where title = 'Single Number II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Bit Manipulation');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'The value 2 appears three times and 3 appears once, so 3 is the answer.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Single Number II' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, '0 and 1 each appear three times. Only 99 appears a single time.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Single Number II' and tests.test_number = 1 and tests.hidden = false;

-- Remove Duplicates from Sorted List II
insert into problem_tags (problem_id, name) select id, 'Linked Lists' from problems where title = 'Remove Duplicates from Sorted List II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Linked Lists');
insert into problem_tags (problem_id, name) select id, 'Two Pointers' from problems where title = 'Remove Duplicates from Sorted List II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Two Pointers');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, '3 and 4 each appear more than once, so every copy of them is removed. The values 1, 2, and 5 appear once and stay.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Remove Duplicates from Sorted List II' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, '1 appears three times, so all three copies are removed. 2 and 3 are left.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Remove Duplicates from Sorted List II' and tests.test_number = 1 and tests.hidden = false;

-- Integer to Roman
insert into problem_tags (problem_id, name) select id, 'Strings' from problems where title = 'Integer to Roman'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Strings');
insert into problem_tags (problem_id, name) select id, 'Math' from problems where title = 'Integer to Roman'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Math');
insert into problem_tags (problem_id, name) select id, 'Hash Tables' from problems where title = 'Integer to Roman'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Hash Tables');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, '58 = 50 + 5 + 3, written as L for 50, V for 5, and III for 3.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Integer to Roman' and tests.test_number = 1 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, '1994 = 1000 + 900 + 90 + 4, written as M, CM, XC, and IV. A smaller symbol before a larger one is subtracted.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Integer to Roman' and tests.test_number = 2 and tests.hidden = false;

-- Best Time to Buy and Sell Stock II
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Best Time to Buy and Sell Stock II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Greedy' from problems where title = 'Best Time to Buy and Sell Stock II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Greedy');
insert into problem_tags (problem_id, name) select id, 'Dynamic Programming' from problems where title = 'Best Time to Buy and Sell Stock II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Dynamic Programming');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'Buy at 1 and sell at 5 for 4, then buy at 3 and sell at 6 for 3. The total profit is 7.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Best Time to Buy and Sell Stock II' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'The price never rises from one day to the next, so there is no profitable trade and the answer is 0.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Best Time to Buy and Sell Stock II' and tests.test_number = 1 and tests.hidden = false;

-- Pascal's Triangle
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Pascal''s Triangle'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Dynamic Programming' from problems where title = 'Pascal''s Triangle'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Dynamic Programming');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'Each row starts and ends with 1. The 2 in the third row is the sum of the two 1s above it.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Pascal''s Triangle' and tests.test_number = 2 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'Every inner number is the sum of the two numbers above it. For example, in the last row 6 = 3 + 3 and 4 = 1 + 3.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Pascal''s Triangle' and tests.test_number = 3 and tests.hidden = false;

-- Pascal's Triangle II
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Pascal''s Triangle II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Dynamic Programming' from problems where title = 'Pascal''s Triangle II'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Dynamic Programming');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'Rows are counted from 0, so row 3 is the fourth row: 1, 3, 3, 1. Each 3 is 1 + 2 from the row above.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Pascal''s Triangle II' and tests.test_number = 2 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'Row 1 is the second row of the triangle, which is two 1s.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Pascal''s Triangle II' and tests.test_number = 1 and tests.hidden = false;

-- Roman to Integer
insert into problem_tags (problem_id, name) select id, 'Strings' from problems where title = 'Roman to Integer'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Strings');
insert into problem_tags (problem_id, name) select id, 'Math' from problems where title = 'Roman to Integer'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Math');
insert into problem_tags (problem_id, name) select id, 'Hash Tables' from problems where title = 'Roman to Integer'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Hash Tables');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'L is 50, V is 5, and III is 3. Adding them gives 58.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Roman to Integer' and tests.test_number = 1 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'M is 1000, CM is 900, XC is 90, and IV is 4. A smaller symbol before a larger one is subtracted, so the total is 1994.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Roman to Integer' and tests.test_number = 2 and tests.hidden = false;

-- Remove Duplicates from Sorted List
insert into problem_tags (problem_id, name) select id, 'Linked Lists' from problems where title = 'Remove Duplicates from Sorted List'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Linked Lists');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'The value 1 appears twice. One copy is kept, so the result is 1 followed by 2.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Remove Duplicates from Sorted List' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'Both 1 and 3 are repeated. Keeping one copy of each value leaves 1, 2, 3.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Remove Duplicates from Sorted List' and tests.test_number = 1 and tests.hidden = false;

-- Single Number
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Single Number'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Bit Manipulation' from problems where title = 'Single Number'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Bit Manipulation');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, '2 appears twice and 1 appears once, so the answer is 1.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Single Number' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, '1 and 2 each appear twice. Only 4 has no partner.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Single Number' and tests.test_number = 1 and tests.hidden = false;

-- Best Time to Buy and Sell Stock
insert into problem_tags (problem_id, name) select id, 'Arrays' from problems where title = 'Best Time to Buy and Sell Stock'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Arrays');
insert into problem_tags (problem_id, name) select id, 'Dynamic Programming' from problems where title = 'Best Time to Buy and Sell Stock'
    and not exists (select 1 from problem_tags where problem_id = problems.id and name = 'Dynamic Programming');
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 0, tests.input, tests.expected_output, 'Buy at the lowest price, 1, and sell later at 6 for a profit of 5. Selling at 7 is not possible because that price comes before the purchase.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Best Time to Buy and Sell Stock' and tests.test_number = 0 and tests.hidden = false;
insert into problem_examples (problem_id, example_number, input, expected_output, explanation)
    select tests.problem_id, 1, tests.input, tests.expected_output, 'The price only falls, so no purchase can be sold for more. The best choice is not to trade, for a profit of 0.'
    from tests join problems on problems.id = tests.problem_id
    where problems.title = 'Best Time to Buy and Sell Stock' and tests.test_number = 1 and tests.hidden = false;

-- The examples above replace the hand-written tables in these descriptions.
-- Only a description that still has an Examples section followed by a
-- Constraints section is changed, and only once both examples exist.
update problems
set description = substr(description, 1, instr(description, '### Examples') - 1)
    || substr(description, instr(description, '### Constraints'))
where instr(description, '### Examples') > 0
  and instr(description, '### Constraints') > instr(description, '### Examples')
  and (select count(*) from problem_examples where problem_examples.problem_id = problems.id) = 2;

-- This sentence lived in the removed Examples section and is not an example.
update problems
set description = replace(description, '### Constraints',
    'By convention `n = 0` has exactly one partition, the empty sum, so the answer for `0` is `1`.'
    || char(10) || char(10) || '### Constraints')
where title = 'Integer Partitions'
  and instr(description, 'By convention') = 0
  and instr(description, '### Examples') = 0;
