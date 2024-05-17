#!/bin/bash

CW_DAY="$(date +%u)" # Current Weekday number: 1-7
MOD=''
if [[ "$1" == 'last' ]]; then
  # Modifier to register worklogs for last week.
  MOD='-7 days'
fi

get_date() {
  if [[ "$CW_DAY" -gt "$2" ]]; then
    date -d "last-${1}${MOD}" '+%Y-%m-%d'
  else
    date -d "${1}${MOD}" '+%Y-%m-%d'
  fi
}

MON_DATE="$(get_date 'monday' 1)"
TUE_DATE="$(get_date 'tuesday' 2)"
WED_DATE="$(get_date 'wednesday' 3)"
THU_DATE="$(get_date 'thursday' 4)"
FRI_DATE="$(get_date 'friday' 5)"
SAT_DATE="$(get_date 'saturday' 6)"
SUN_DATE="$(get_date 'sunday' 7)"

# Create worklogs
jig log --date "$MON_DATE" 30 JB-129 # Dedicated issue for SCRUM meetings or similar.
jig log --date "$TUE_DATE" 30 JB-129
jig log --date "$WED_DATE" 30 JB-129
jig log --date "$THU_DATE" 30 JB-129
jig log --date "$FRI_DATE" 1h JB-129
