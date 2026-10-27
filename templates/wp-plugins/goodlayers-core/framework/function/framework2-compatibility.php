<?php

	if( !function_exists('gdlr_core_fw2_convert_pb') ){
		function gdlr_core_fw2_convert_pb( $page_builder ){

			$ret = array();

			foreach( $page_builder as $item ){
				
				$pb_item = array();

				if( !empty($item['item-type']) ){

					// wrapper
					if( $item['item-type'] == 'wrapper' ){
						
						$pb_item['template'] = 'wrapper';
						$pb_item['value'] = empty($item['option'])? array(): $item['option'];

						if( $item['type'] == 'full-size-wrapper' ){
							$pb_item['type'] = 'background';
							$pb_item['value']['content-layout'] = 'full';

						}else if( $item['type'] == 'parallax-bg-wrapper' ){
							$pb_item['type'] = 'background';
							$pb_item['value']['content-layout'] = 'boxed';
							$pb_item['value']['background-type'] = empty($pb_item['value']['type'])? 'image': $pb_item['value']['type'];
							$pb_item['value']['background-image'] = empty($pb_item['value']['background'])? '': $pb_item['value']['background'];
							$pb_item['value']['background-video-url'] = empty($pb_item['value']['video'])? '': $pb_item['value']['video'];

						}else if( $item['type'] == 'color-wrapper' ){
							$pb_item['type'] = 'background';
							$pb_item['value']['content-layout'] = 'boxed';
							$pb_item['value']['background-type'] = 'color';
							$pb_item['value']['background-color'] = empty($pb_item['value']['background'])? '': $pb_item['value']['background'];
							unset($pb_item['value']['background']);

						}else if( strpos($item['type'], 'column') !== false && preg_match('/^(\d)\/(\d)$/', $item['size'], $matches) ){
							$pb_item['type'] = 'column';
							$pb_item['column'] = 60 * intval($matches[1]) / intval($matches[2]);
						}

						// hide wrapper in
						if( !empty($pb_item['value']['show-section']) ){
							if( strpos($pb_item['value']['show-section'], 'hide-in-tablet-mobile') !== false ){
								$pb_item['value']['hide-this-wrapper-in'] = 'tablet-mobile';
							}else if( strpos($pb_item['value']['show-section'], 'hide-in-tablet') !== false ){
								$pb_item['value']['hide-this-wrapper-in'] = 'tablet';
							}else if( strpos($pb_item['value']['show-section'], 'hide-in-mobile') !== false ){
								$pb_item['value']['hide-this-wrapper-in'] = 'mobile';
							}else{
								$pb_item['value']['hide-this-wrapper-in'] = 'none';
							}
							unset($pb_item['value']['show-section']);
						}


						// item inside container
						if( !empty($item['items']) ){
							$pb_item['items'] = gdlr_core_fw2_convert_pb($item['items']);
						}

					// item
					}else if( $item['item-type'] == 'item' ){

						$pb_item['template'] = 'element';
						$pb_item['type'] = $item['type'];

						// change item name
						switch( $pb_item['type'] ){
							case 'styled-box':
							case 'box-icon-item':
								$pb_item['type'] = 'feature-box';
								break;
							case 'banner':
							case 'slider':
								$pb_item['type'] = 'gallery';
							case 'content':
								$pb_item['type'] = 'text-box';
								break;
							case 'feature-media':
								$pb_item['type'] = 'promo-box';
								break;
							case 'image-frame':
								$pb_item['type'] = 'image';
								break;
							case 'quote':
								$pb_item['type'] = 'blockquote';
								break;
							case 'skill-item':
								$pb_item['type'] = 'counter';
								break;
							case 'notification':
								$pb_item['type'] = 'alert-box';
								break;
							case 'pie-chart':
								$pb_item['type'] = 'skill-circle';
								break;
						}

						// item options
						if( !empty($item['option']) ){
							$pb_item['value'] = gdlr_core_fw2_convert_option($item['option'], $pb_item['type']);
						}

					}
					
				}

				$ret[] = $pb_item;

			}

			return $ret;
		}
	}

	// for option compatibility
	if( !function_exists('gdlr_core_fw2_convert_option') ){
		function gdlr_core_fw2_convert_option( $option, $type ){

			switch( $type ){

				case 'accordion':
					if( !empty($option['accordion']) ){
						$accordions = json_decode($option['accordion'], true);
						if( !empty($accordions) ){

							$option['tabs'] = array();
							foreach( $accordions as $accordion ){
								$option['tabs'][] = array(
									'title' => empty($accordion['gdl-tab-title'])? '': $accordion['gdl-tab-title'],
									'content' => empty($accordion['gdl-tab-content'])? '': $accordion['gdl-tab-content']
								);
							}
						}
						unset($option['accordion']);
						$option['style'] = 'box-icon';
					}
					break;

				case 'blockquote':
					$option['text-align'] = 'center';
					break;

				case 'blog':
					$option['layout'] = empty($option['blog-layout'])? '': $option['blog-layout'];
					unset($option['blog-layout']);

					switch($option['blog-style']){
						case 'blog-widget-1-1': 
						case 'blog-widget-1-2': 
						case 'blog-widget-1-3': 
						case 'blog-widget-1-4': 
							$option['blog-style'] = 'blog-widget';
							break;

						case 'blog-1-1': 
							$option['blog-style'] = 'blog-column';
							$option['column-size'] = 60;
							break;
						case 'blog-1-2': 
							$option['blog-style'] = 'blog-column';
							$option['column-size'] = 30;
							break;
						case 'blog-1-3': 
							$option['blog-style'] = 'blog-column';
							$option['column-size'] = 20;
							break;
						case 'blog-1-4': 
							$option['blog-style'] = 'blog-column';
							$option['column-size'] = 15;
							break;

						case 'blog-medium': 
							$option['blog-style'] = 'blog-left-thumbnail';
							break;

						case 'blog-full': 
							break;

						default:
							$option['blog-style'] = 'blog-full';
							break;
					}

					break; 

				case 'counter':
					$option['start-number'] = 0;
					$option['end-number'] = empty($option['title'])? '': $option['title'];
					$option['suffix'] = '';
					$option['divider'] = 'enable';
					$option['bottom-text'] = empty($option['caption'])? '': $option['caption'];
					break;

				case 'divider':
					$options['style'] = empty($option['type'])? '': $option['type'];
					$option['type'] = 'normal';
					$options['divider-width'] = empty($option['size'])? '': $option['size'];
					unset($option['size']);
					break;

				case 'feature-box':
					$option['caption'] = '';
					$option['text-align'] = 'center';
					$option['icon'] = empty($option['icon'])? '': $option['icon'];
					$option['title'] = empty($option['title'])? '': $option['title'];
					break;

				case 'gallery':
					$option['column'] = empty($option['gallery-columns'])? 3: $option['gallery-columns'];
					unset($option['gallery-columns']);

					if( !empty($option['slider']) ){
						$gallery = json_decode($option['slider'], true);
						if( !empty($gallery[0]) ){
							$option['gallery'] = array();
							foreach( $gallery[0] as $image_id ){
								$option['gallery'][] = array(
									'id' => $image_id
								);
							}
						}
						unset($option['slider']);
					}

					if( !empty($option['banner-columns']) ){
						$option['column'] = $option['banner-columns'];
						$option['style'] = 'grid';
						$option['layout'] = 'carousel';
						
						unset($option['banner-columns']);
					}else if( !empty($option['slider-type']) ){
						$option['style'] = 'slider';

						unset($option['slider-type']);
					}
					break;

				case 'image':
					$option['image'] = empty($option['image-id'])? '': $option['image-id'];
					unset($option['image-id']);
					break;

				case 'notification':
					$option['title'] = '';
					break;

				case 'portfolio':
					$option['layout'] = empty($option['portfolio-layout'])? '': $option['portfolio-layout'];
					unset($option['portfolio-layout']);
					
					if( !empty($option['portfolio-style']) ){
						if( strpos($option['portfolio-style'], 'classic') !== false ){
							$option['portfolio-style'] = 'grid';
						}else if( strpos($option['portfolio-style'], 'modern') !== false ){
							$option['portfolio-style'] = 'modern';
						}else{
							$option['portfolio-style'] = 'grid';
						}
					}

					break; 

				case 'price-table':
					if( !empty($option['price-table']) ){
						$price_table = json_decode($option['price-table'], true);
						if( !empty($price_table) ){

							$option['tabs'] = array();
							foreach( $price_table as $tab ){
								$option['tabs'][] = array(
									'title' => empty($tab['gdl-tab-title'])? '': $tab['gdl-tab-title'],
									'price' => empty($tab['gdl-tab-price'])? '': $tab['gdl-tab-price'],
									'content' => empty($tab['gdl-tab-content'])? '': $tab['gdl-tab-content'],
									'feature-price' => (empty($tab['gdl-tab-active']) || $tab['gdl-tab-active'] == 'no')? 'disable': 'enable',
									'button-link' => empty($tab['gdl-tab-button-link'])? '': $tab['gdl-tab-button-link'],
								);
							}
						}
						unset($option['price-table']);
					}
					break;

				case 'product':
					if( !empty($option['portfolio-layout']) ){
						$option['layout'] = empty($option['portfolio-layout'])? '': $option['portfolio-layout'];
						unset($option['portfolio-layout']);
					}else if( !empty($option['product-layout']) ){
						$option['layout'] = empty($option['product-layout'])? '': $option['product-layout'];
						unset($option['product-layout']);
					}
					break; 

				case 'column-service':
					$option['media-type'] = empty($option['type'])? 'icon': $option['type']; 
					unset($option['type']);
					$option['style'] = 'center_icon-top';

					if( !empty($option['learn-more-text']) ){
						$option['read-more-text'] = $option['learn-more-text'];
						$option['read-more-link'] = empty($option['learn-more-link'])? '': $option['learn-more-link'];
					}

					break;

				case 'skill-bar': 
					$option['bar-size'] = 'small';
					$option['bar-type'] = 'rectangle';
					$option['tabs'] = array(
						array(
							'heading-text' => empty($option['content'])? '': $option['content'],
							'icon' => empty($option['icon'])? '': $option['icon'],
							'percent' => empty($option['percent'])? '': $option['percent'],
							'bar-text' => ''
						)
					);
					unset($option['content']);
					unset($option['icon']);
					unset($option['percent']);
					break;

				case 'skill-circle': 
					$option['item-size'] = 'large';
					$option['heading-text'] = empty($option['title'])? '': $option['title'];
					$option['bar-text'] = empty($option['content'])? '': $option['content'];
					$option['percent'] = empty($option['progress'])? '': $option['progress'];
					unset($option['title']);
					unset($option['content']);
					unset($option['progress']);
					break;

				case 'stunning-text': 
					$option['content'] = empty($option['caption'])? '': $option['caption'];
					unset($option['caption']);
					break;

				case 'tab':
					if( !empty($option['tab']) ){
						$tabs = json_decode($option['tab'], true);
						if( !empty($tabs) ){

							$option['tabs'] = array();
							foreach( $tabs as $tab ){
								$option['tabs'][] = array(
									'title' => empty($tab['gdl-tab-title'])? '': $tab['gdl-tab-title'],
									'content' => empty($tab['gdl-tab-content'])? '': $tab['gdl-tab-content']
								);
							}
						}
						unset($option['tab']);

						if( $option['style'] == 'horizontal' ){
							$option['style'] = 'style1-horizontal';
						}else if( $option['style'] == 'vertical' || $option['style'] == 'vertical right' ){
							$option['style'] = 'style1-vertical';
						}else{
							$option['style'] = 'style1-horizontal';
						}
					}
					break;

				case 'testimonial': 
					$option['style'] = 'left';
					$option['column'] = empty($option['testimonial-columns'])? 3: $option['testimonial-columns']; 
					unset($option['testimonial-columns']);
					$option['carousel'] = (!empty($option['testimonial-type']) && $option['testimonial-type'] == 'carousel')? 'enable': 'disable'; 
					unset($option['testimonial-columns']);

					$option['testimonial'] = json_decode($option['testimonial'], true);
					$option['tabs'] = array();

					if( !empty($option['testimonial']) ){
						foreach( $option['testimonial'] as $tab ){
							$new_tab = array();

							$new_tab['title'] = empty($tab['gdl-tab-title'])? '': $tab['gdl-tab-title'];
							$new_tab['position'] = empty($tab['gdl-tab-position'])? '': $tab['gdl-tab-position'];
							$new_tab['content'] = empty($tab['gdl-tab-content'])? '': $tab['gdl-tab-content'];
							$new_tab['image'] = empty($tab['gdl-tab-author-image'])? '': $tab['gdl-tab-author-image'];
							$new_tab['image-img'] = empty($tab['gdl-tab-author-image-url'])? '': $tab['gdl-tab-author-image-url'];

							$option['tabs'][] = $new_tab;
						}
					}
					
					unset($option['testimonial']);
					break;

				case 'toggle-box':
					if( !empty($option['toggle-box']) ){
						$toggle_boxes = json_decode($option['toggle-box'], true);
						if( !empty($toggle_boxes) ){

							$option['tabs'] = array();
							foreach( $toggle_boxes as $toggle_box ){
								$option['tabs'][] = array(
									'title' => empty($toggle_box['gdl-tab-title'])? '': $toggle_box['gdl-tab-title'],
									'content' => empty($toggle_box['gdl-tab-content'])? '': $toggle_box['gdl-tab-content'],
									'active' => (empty($toggle_box['gdl-tab-active']) || $toggle_box['gdl-tab-active'] == 'no')? 'disable': 'enable'
								);
							}
						}
						unset($option['toggle-box']);
						$option['style'] = 'box-icon';
					}
					break;

				case 'video':
					$option['video-url'] = empty($option['url'])? '': $option['url'];
					unset($option['url']);
					break;

				/* slider plugin */
				case 'master-slider':
					if( empty($option['master-slider-id']) ){ 
						$option['master-slider-id'] = empty($option['id'])? '': $option['id'];
					}
					break;

				case 'layer-slider':
					if( empty($option['layer-slider-id']) ){ 
						$option['layer-slider-id'] = empty($option['id'])? '': $option['id'];
					}
					break;

				case 'revolution-slider':
					if( empty($option['revolution-slider-id']) ){ 
						$option['revolution-slider-id'] = empty($option['id'])? '': $option['id'];
					}
					break;
			}

			// convert icon
			if( !empty($option['icon']) ){
				$option['icon'] = 'fa ' . str_replace('icon-', 'fa-', $option['icon']);
			}

			// convert category/tag
			if( !empty($option['category']) ){
				$option['category'] = explode(',', $option['category']);
			}
			if( !empty($option['tag']) ){
				$option['tag'] = explode(',', $option['tag']);
			}

			// convert excerpt
			if( !empty($option['num-excerpt']) ){
				$option['excerpt'] = 'specify-number';
				$option['excerpt-number'] = $option['num-excerpt'];
				unset($option['num-excerpt']);
			}

			return $option;
		}
	}	

	// decode page builder data
	if( !function_exists('gdlr_core_fw2_decode_preventslashes') ){
		function gdlr_core_fw2_decode_preventslashes( $value ){
			$value = str_replace('|gq6|', '\\\\\\"', $value);
			$value = str_replace('|gq5|', '\\\\\"', $value);
			$value = str_replace('|gq4|', '\\\\"', $value);
			$value = str_replace('|gq3|', '\\\"', $value);
			$value = str_replace('|gq2|', '\\"', $value);
			$value = str_replace('|gq"|', '\"', $value);
			$value = str_replace('|g2t|', '\\\t', $value);
			$value = str_replace('|g1t|', '\t', $value);			
			$value = str_replace('|g2n|', '\\\n', $value);
			$value = str_replace('|g1n|', '\n', $value);
			return $value;
		}
	}	

	// convert old data to new one
	add_filter('gdlr_core_page_builder_val', 'gdlr_core_page_builder_val_fw2_compatibility');
	if( !function_exists('gdlr_core_page_builder_val_fw2_compatibility') ){
		function gdlr_core_page_builder_val_fw2_compatibility( $page_builder_val ){

			if( empty($page_builder_val) ){
				$page_builder_val = array();

				$above_sidebar = json_decode(gdlr_core_fw2_decode_preventslashes(get_post_meta(get_the_ID(), 'above-sidebar', true)), true);
				if( !empty($above_sidebar) ){
					$page_builder_val = array_merge($page_builder_val, gdlr_core_fw2_convert_pb($above_sidebar));
				}

				$content_with_sidebar = json_decode(gdlr_core_fw2_decode_preventslashes(get_post_meta(get_the_ID(), 'content-with-sidebar', true)), true);
				if( !empty($content_with_sidebar) ){
					$page_builder_val = array_merge($page_builder_val, gdlr_core_fw2_convert_pb($content_with_sidebar));
				}

				$below_sidebar = json_decode(gdlr_core_fw2_decode_preventslashes(get_post_meta(get_the_ID(), 'below-sidebar', true)), true);
				if( !empty($below_sidebar) ){
					$page_builder_val = array_merge($page_builder_val, gdlr_core_fw2_convert_pb($below_sidebar));
				}

			}

			return $page_builder_val;

		}
	}