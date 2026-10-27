<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	gdlr_core_page_builder_element::add_element('product-category', 'gdlr_core_pb_element_product_category'); 
	
	if( !class_exists('gdlr_core_pb_element_product_category') ){
		class gdlr_core_pb_element_product_category{
			
			// get the element settings
			static function get_settings(){
				return array(
					'icon' => 'fa-shopping-cart',
					'title' => esc_html__('Product Category (Woocommerce)', 'goodlayers-core')
				);
			}
			
			// return the element options
			static function get_options(){
				global $gdlr_core_item_pdb;
				
				return apply_filters('gdlr_core_product_category_item_options', array(					
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'category' => array(
								'title' => esc_html__('Category', 'goodlayers-core'),
								'type' => 'multi-combobox',
								'options' => gdlr_core_get_term_list('product_cat'),
								'description' => esc_html__('You can use Ctrl/Command button to select multiple items or remove the selected item. Leave this field blank to select all items in the list.', 'goodlayers-core'),
							),
						),
					),
					'settings' => array(
						'title' => esc_html__('Product Style', 'goodlayers-core'),
						'options' => array(
							'column-size' => array(
								'title' => esc_html__('Column Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array( 60 => 1, 30 => 2, 20 => 3, 15 => 4, 12 => 5 ),
								'default' => 20,
							),
							'thumbnail-size' => array(
								'title' => esc_html__('Thumbnail Size', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'thumbnail-size',
							),
						),
					),	
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'padding-bottom' => array(
								'title' => esc_html__('Padding Bottom', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => $gdlr_core_item_pdb
							),
						),
					)
				));
			}

			// get the preview for page builder
			static function get_preview( $settings = array() ){
				$content  = self::get_content($settings, true);	
				return $content;
			}			
			
			// get the content from settings
			static function get_content( $settings = array(), $preview = false ){
				global $gdlr_core_item_pdb;
				
				// default variable
				if( empty($settings) ){
					$settings = array(
						'category' => '', 'padding-bottom' => $gdlr_core_item_pdb
					);
				}

				$settings['column-size'] = empty($settings['column-size'])? 20: intval($settings['column-size']);
				$settings['thumbnail-size'] = empty($settings['thumbnail-size'])? 'full': $settings['thumbnail-size'];

				// start printing item
				$ret  = '<div class="gdlr-core-product-category-item gdlr-core-item-pdb clearfix" ';
				if( !empty($settings['padding-bottom']) && $settings['padding-bottom'] != $gdlr_core_item_pdb ){
					$ret .= gdlr_core_esc_style(array('padding-bottom'=>$settings['padding-bottom']));
				}
				if( !empty($settings['id']) ){
					$ret .= ' id="' . esc_attr($settings['id']) . '" ';
				}
				$ret .= ' >';

				if( class_exists('WooCommerce') ){

					if( !empty($settings['category']) ){
						
						$column_sum = 0;

						foreach( $settings['category'] as $cat_name ){
							$term = get_term_by('slug', $cat_name, 'product_cat');
		
							if( !empty($term) ){

								if( !empty($settings['column-size']) ){
									$item_list_class = empty($settings['column-size'])? '': ' gdlr-core-column-' . $settings['column-size'];
								}
								if( $column_sum == 0 || $column_sum + intval($settings['column-size']) > 60 ){
									$column_sum = intval($settings['column-size']);
									$item_list_class .= ' gdlr-core-column-first';
								}else{
									$column_sum += intval($settings['column-size']);
								}

								$thumbnail_id = get_term_meta( $term->term_id, 'thumbnail_id', true );
								
								$ret .= '<div class="gdlr-core-item-list gdlr-core-item-pdlr ' . esc_attr($item_list_class) . '" >';
								$ret .= '<div class="gdlr-core-product-category-grid" >';
								if( !empty($thumbnail_id) ){
									$ret .= '<div class="gdlr-core-product-category-thumbnail gdlr-core-media-image" >';
									$ret .= wp_get_attachment_image($thumbnail_id, $settings['thumbnail-size']);
									$ret .= '</div>';
								}
								$ret .= '<h3 class="gdlr-core-product-category-title"><a href="' . esc_url(get_term_link($term)) . '" >' . $term->name . '</a></h3>';
								$ret .= '<a class="gdlr-core-product-category-read-more" href="' . esc_url(get_term_link($term)) . '" >' . esc_html__('Shop Now', 'goodlayers-core') . '</a>';
								$ret .= '</div>';
								$ret .= '</div>';
							}
						}
					}

				}else{
					$ret .= '<div class="gdlr-core-external-plugin-message">' . esc_html__('Please install and activate the "Woocommerce" plugin before using this item.', 'goodlayers-core') . '</div>';
				}
				
				$ret .= '</div>'; // gdlr-core-product-item

				return $ret;
			}			
			
		} // gdlr_core_pb_element_product_category
	} // class_exists	