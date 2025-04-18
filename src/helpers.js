function splitString(string) {
  return string.trim().split(' ');
}

export function getMatcher(styles, prefer) {
  if (typeof classNames !== 'string') {
    return null;
  }

  let globalClassNames = [];
  let localClassNames = [];
  let restClassNames = splitString(
    classNames
      .replace(/\s{2,}/g, ' ')
      .replace(/:global\([\s\S]*?\)/g, text => {
        globalClassNames = globalClassNames.concat(
          splitString(text.replace(/(:global\(|\))/g, ''))
        );
        return '';
      })
      .replace(/:local\([\s\S]*?\)/g, text => {
        localClassNames = localClassNames.concat(
          splitString(text.replace(/(:local\(|\))/g, ''))
        );
        return '';
      })
  );

  if (prefer === 'local') {
    localClassNames = localClassNames.concat(restClassNames);
  } else {
    globalClassNames = globalClassNames.concat(restClassNames);
  }

  return localClassNames
    .map(className => styles[className] || className)
    .concat(globalClassNames)    
    .join(' ')
    .trim();
}
