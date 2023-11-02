#!/bin/bash

# Modifier to register worklogs for last week.
MOD=''
if [[ "$1" == 'last' ]]; then
  MOD='-7 days'
fi

CW_DAY="$(date +%u)" # Current Weekday number: 1-7

get_date() {
  date -d "${1}${MOD}" '+%Y-%m-%d' # Format: 2023-08-20
}

# Monday
if [[ $CW_DAY -gt 1 ]]; then
  MON_DATE="$(get_date 'last-monday')"
else
  MON_DATE="$(get_date 'monday')"
fi
# Tuesday
if [[ $CW_DAY -gt 2 ]]; then
  TUE_DATE="$(get_date 'last-tuesday')"
else
  TUE_DATE="$(get_date 'tuesday')"
fi
# Wednesday
if [[ $CW_DAY -gt 3 ]]; then
  WED_DATE="$(get_date 'last-wednesday')"
else
  WED_DATE="$(get_date 'wednesday')"
fi
# Thursday
if [[ $CW_DAY -gt 4 ]]; then
  THU_DATE="$(get_date 'last-thursday')"
else
  THU_DATE="$(get_date 'thursday')"
fi
# Friday
if [[ $CW_DAY -gt 5 ]]; then
  FRI_DATE="$(get_date 'last-friday')"
else
  FRI_DATE="$(get_date 'friday')"
fi
# Saturday
if [[ $CW_DAY -gt 6 ]]; then
  SAT_DATE="$(get_date 'last-saturday')"
else
  SAT_DATE="$(get_date 'saturday')"
fi
# Sunday
# No switch as CW_DAY is never greater than 7
SUN_DATE="$(get_date 'sunday')"

# Actually create worklogs
jig log --date "$MON_DATE" 30 JB-129 # Dedicated jira issue for SCRUM meetings or similar.
# jig log --date "$TUE_DATE" 30 JB-129
jig log --date "$WED_DATE" 30 JB-129
# jig log --date "$THU_DATE" 30 JB-129
jig log --date "$FRI_DATE" 1h JB-129
