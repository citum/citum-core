'use strict';

function locatorValueToString(value) {
  if (typeof value === 'string') {
    return value;
  }

  if (value && typeof value === 'object' && typeof value.value === 'string') {
    return value.value;
  }

  return '';
}

function locatorSegmentToString(segment) {
  const value = locatorValueToString(segment.value);
  return value ? `${segment.label} ${value}` : segment.label;
}

function toCiteprocItem(item, suppressAuthor) {
  if (!item.locator || typeof item.locator !== 'object') {
    return {
      id: item.id,
      locator: item.locator,
      label: item.label,
      prefix: item.prefix,
      suffix: item.suffix,
      'suppress-author': suppressAuthor,
    };
  }

  if (Array.isArray(item.locator.segments)) {
    return {
      id: item.id,
      locator: item.locator.segments.map(locatorSegmentToString).join(', '),
      prefix: item.prefix,
      suffix: item.suffix,
      'suppress-author': suppressAuthor,
    };
  }

  return {
    id: item.id,
    locator: locatorValueToString(item.locator.value),
    label: item.locator.label,
    prefix: item.prefix,
    suffix: item.suffix,
    'suppress-author': suppressAuthor,
  };
}

module.exports = {
  locatorSegmentToString,
  locatorValueToString,
  toCiteprocItem,
};
